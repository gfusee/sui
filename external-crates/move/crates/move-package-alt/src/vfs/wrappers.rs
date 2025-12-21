use std::cmp::Ordering;
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use fs4::fs_std::FileExt;
use vfs::{FileSystem, PhysicalFS, SeekAndRead, VfsMetadata, VfsPath, VfsResult};

pub trait FileSystemExt: FileSystem {
    fn open_lockable(&self, path: &str, should_truncate: bool) -> VfsResult<Lockable>;
}

pub trait Lock {
    fn lock_exclusive(& self) -> std::io::Result<()>;
    fn unlock(& self) -> std::io::Result<()>;
}

pub trait LockableAndDebuggable: Lock + Debug {}

#[derive(Clone, Debug)]
pub struct ArcFileSystem {
    fs: Arc<Box<dyn FileSystemExt>>
}

#[derive(Debug)]
pub struct Lockable {
    inner: Box<dyn LockableAndDebuggable>
}

impl Lock for Lockable {
    fn lock_exclusive(&self) -> std::io::Result<()> {
        self.inner.lock_exclusive()
    }

    fn unlock(&self) -> std::io::Result<()> {
        self.inner.unlock()
    }
}

impl Lock for File {
    fn lock_exclusive(&self) -> std::io::Result<()> {
       fs4::fs_std::FileExt::lock_exclusive(self)
    }

    fn unlock(&self) -> std::io::Result<()> {
        fs4::fs_std::FileExt::unlock(self)
    }
}

impl FileSystemExt for PhysicalFS {
    fn open_lockable(&self, path: &str, should_truncate: bool) -> VfsResult<Lockable> {
        let lock = OpenOptions::new()
            .truncate(should_truncate)
            .write(true)
            .read(true)
            .create(true)
            .open(&path)?;

        let lockable = Lockable { inner: Box::new(lock) };

        Ok(lockable)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VirtualPath {
    path: VfsPath,
    filesystem: ArcFileSystem
}

impl FileSystem for ArcFileSystem {
    fn read_dir(&self, path: &str) -> VfsResult<Box<dyn Iterator<Item=String> + Send>> {
        self.fs.read_dir(path)
    }

    fn create_dir(&self, path: &str) -> VfsResult<()> {
        self.fs.create_dir(path)
    }

    fn open_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndRead + Send>> {
        self.fs.open_file(path)
    }

    fn create_file(&self, path: &str) -> VfsResult<Box<dyn Write + Send>> {
        self.fs.create_file(path)
    }

    fn append_file(&self, path: &str) -> VfsResult<Box<dyn Write + Send>> {
        self.fs.append_file(path)
    }

    fn metadata(&self, path: &str) -> VfsResult<VfsMetadata> {
        self.fs.metadata(path)
    }

    fn exists(&self, path: &str) -> VfsResult<bool> {
        self.fs.exists(path)
    }

    fn remove_file(&self, path: &str) -> VfsResult<()> {
        self.fs.remove_file(path)
    }

    fn remove_dir(&self, path: &str) -> VfsResult<()> {
        self.fs.remove_dir(path)
    }

    fn copy_file(&self, src: &str, dest: &str) -> VfsResult<()> {
        self.fs.copy_file(src, dest)
    }

    fn move_file(&self, src: &str, dest: &str) -> VfsResult<()> {
        self.fs.move_file(src, dest)
    }

    fn move_dir(&self, src: &str, dest: &str) -> VfsResult<()> {
        self.fs.move_dir(src, dest)
    }
}

impl FileSystemExt for ArcFileSystem {
    fn open_lockable(&self, path: &str, should_truncate: bool) -> VfsResult<Lockable> {
        self.fs.open_lockable(path, should_truncate)
    }
}

impl PartialEq for VirtualPath {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl Eq for VirtualPath {}

impl VirtualPath {
    pub(crate) fn new<FS: FileSystemExt>(
        filesystem: FS
    ) -> Self {
        Self {
            path: VfsPath::new(filesystem.clone()),
            filesystem: ArcFileSystem {
                fs: Arc::new(Box::new(filesystem)),
            },
        }
    }

    pub(crate) fn join(&self, path: impl AsRef<str>) -> VfsResult<Self> {
        self.path.join(path).map(|e| self.with_vfs_path(e))
    }

    pub(crate) fn root(&self) -> Self {
        self.with_vfs_path(self.path.root())
    }

    pub(crate) fn as_str(&self) -> &str {
        self.path.as_str()
    }

    pub(crate) fn is_dir(&self) -> VfsResult<bool> {
        self.path.is_dir()
    }

    pub(crate) fn is_file(&self) -> VfsResult<bool> {
        self.path.is_file()
    }

    pub(crate) fn exists(&self) -> VfsResult<bool> {
        self.path.exists()
    }

    pub(crate) fn parent(&self) -> Self {
        self.with_vfs_path(self.path.parent())
    }

    pub(crate) fn create_dir_all(&self) -> VfsResult<()> {
        self.path.create_dir_all()
    }

    pub(crate) fn create_file(&self) -> VfsResult<Box<dyn Write + Send>> {
        self.path.create_file()
    }

    pub fn read_to_string(&self) -> VfsResult<String> {
        self.path.read_to_string()
    }

    pub(crate) fn remove_dir_all(&self) -> VfsResult<()> {
        self.path.remove_dir_all()
    }

    pub fn read_dir(&self) -> VfsResult<Box<dyn Iterator<Item = VirtualPath> + Send>> {
        self.path
            .read_dir()
            .map(|vfs_paths| vfs_paths.map(|e| self.with_vfs_path(e)).into())
    }

    pub fn metadata(&self) -> VfsResult<VfsMetadata> {
        self.path.metadata()
    }

    pub fn extension(&self) -> Option<String> {
        self.path.extension()
    }

    pub fn open_and_lock_exclusive(&self, should_truncate: bool) -> VfsResult<Lockable> {
        self.filesystem.open_lockable(self.path.as_str(), should_truncate)
    }

    fn with_vfs_path(&self, path: VfsPath) -> Self {
        Self {
            path,
            filesystem: self.filesystem.clone(),
        }
    }
}

impl PartialOrd for VirtualPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.path.as_str().partial_cmp(other.path.as_str())
    }
}

impl Ord for VirtualPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.as_str().cmp(other.path.as_str())
    }
}