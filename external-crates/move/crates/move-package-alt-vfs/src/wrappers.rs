use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use move_symbol_pool::Symbol;
use vfs::{FileSystem, OverlayFS, PhysicalFS, SeekAndRead, VfsMetadata, VfsPath, VfsResult};

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
    inner: Box<dyn LockableAndDebuggable + Send>
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

impl LockableAndDebuggable for File {}

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

impl FileSystemExt for OverlayFS {
    fn open_lockable(&self, _path: &str, _should_truncate: bool) -> VfsResult<Lockable> {
        unimplemented!("Is it needed?")
    }
}

#[derive(Clone, Debug)]
pub struct VirtualPath {
    path: VfsPath,
    cwd: VfsPath,
    filesystem: ArcFileSystem,
}

impl ArcFileSystem {
    fn new<FS: FileSystemExt>(fs: FS) -> Self {
        Self {
            fs: Arc::new(Box::new(fs)),
        }
    }
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

impl AsRef<VfsPath> for VirtualPath {
    fn as_ref(&self) -> &VfsPath {
        &self.path
    }
}

impl VirtualPath {
    pub fn new<FS: FileSystemExt>(
        cwd: Option<impl AsRef<str>>,
        filesystem: FS,
    ) -> VfsResult<Self> {
        let arc_filesystem = ArcFileSystem::new(filesystem);
        let path = VfsPath::new(arc_filesystem.clone());
        let cwd = match cwd {
            Some(cwd) => path.root().join(cwd)?,
            None => path.clone()
        };

        Ok(Self {
            path,
            cwd,
            filesystem: arc_filesystem,
        })
    }

    pub fn physical() -> VfsResult<Self> {
        let physical_filesystem = ArcFileSystem::new(PhysicalFS::new("/"));
        let cwd = std::env::current_dir()
            .ok()
            .map(|e| e.canonicalize().ok())
            .flatten()
            .map(|e| e.to_string_lossy().to_string());

        VirtualPath::new(
            cwd,
            physical_filesystem
        )
    }

    pub fn pop(&mut self) -> VfsResult<bool> {
        let mut self_path_buf = PathBuf::from(self.path.as_str());
        let result = self_path_buf.pop();

        *self = self.join(self_path_buf)?;

        Ok(result)
    }

    pub fn join(&self, path: impl AsRef<Path>) -> VfsResult<Self> {
        let path = path.as_ref();
        if path.is_absolute() {
            self.path.root().join(path.to_string_lossy())
        } else {
            self.path.join(path.to_string_lossy())
        }.map(|e| self.with_vfs_path(e))
    }

    pub fn filename(&self) -> String {
        self.path.filename()
    }

    pub fn open_file(&self) -> VfsResult<Box<dyn SeekAndRead + Send>> {
        self.path.open_file()
    }

    pub fn with_extension<S: AsRef<str>>(&self, extension: S) -> VfsResult<VirtualPath> {
        let path_buf_with_extension = PathBuf::from(self.as_str()).with_extension(extension.as_ref());

        self.join(path_buf_with_extension)
    }

    pub fn root(&self) -> Self {
        self.with_vfs_path(self.path.root())
    }

    pub fn cwd(&self) -> Self {
        self.with_vfs_path(self.cwd.clone())
    }

    pub fn as_str(&self) -> &str {
        self.path.as_str()
    }

    pub fn is_dir(&self) -> VfsResult<bool> {
        self.path.is_dir()
    }

    pub fn is_file(&self) -> VfsResult<bool> {
        self.path.is_file()
    }

    pub fn exists(&self) -> VfsResult<bool> {
        self.path.exists()
    }

    pub fn parent(&self) -> Self {
        self.with_vfs_path(self.path.parent())
    }

    pub fn create_dir_all(&self) -> VfsResult<()> {
        self.path.create_dir_all()
    }

    pub fn create_file(&self) -> VfsResult<Box<dyn Write + Send>> {
        self.path.create_file()
    }

    pub fn copy_file(&self, destination: &VirtualPath) -> VfsResult<()> {
        self.path.copy_file(&destination.path)
    }

    pub fn copy_dir(&self, destination: &VirtualPath) -> VfsResult<u64> {
        self.path.copy_dir(&destination.path)
    }

    pub fn read_to_string(&self) -> VfsResult<String> {
        self.path.read_to_string()
    }

    pub fn remove_file(&self) -> VfsResult<()> {
        self.path.remove_file()
    }

    pub fn remove_dir_all(&self) -> VfsResult<()> {
        self.path.remove_dir_all()
    }

    pub fn read_dir(&self) -> VfsResult<Box<dyn Iterator<Item = VirtualPath> + Send>> {
        let self_clone = self.clone();

        self.path
            .read_dir()
            .map(move |iter| {
                let self_clone = self_clone.clone();
                let mapped_iter = iter.map(move |vsf_path| self_clone.with_vfs_path(vsf_path));
                Box::new(mapped_iter) as Box<dyn Iterator<Item = VirtualPath> + Send>
            })
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

    pub fn with_current_dir(&self, cwd: VirtualPath) -> Self {
        Self {
            path: self.path.clone(),
            cwd: cwd.cwd,
            filesystem: self.filesystem.clone(),
        }
    }

    fn with_vfs_path(&self, path: VfsPath) -> Self {
        Self {
            path,
            cwd: self.cwd.clone(),
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