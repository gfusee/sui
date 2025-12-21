use crate::vfs::errors::TempDirResult;
use std::env;
use uuid::Uuid;
use vfs::VfsPath;

pub(crate) struct TempDir {
    path: VfsPath
}

impl Drop for TempDir {
    fn drop(&mut self) {
        _ = self.path.remove_dir_all();
    }
}

impl TempDir {
    pub fn new(base: VfsPath) -> TempDirResult<Self> {
        let tempdir_name = format!("tmp-{}", Uuid::new_v4());

        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        let os_tempdir = env::temp_dir().canonicalize()?;
        let tmp_base = os_tempdir.to_str().unwrap_or_else(|| "tmp");

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        let tmp_base = "tmp".to_string();

        let tempdir = base.root()
            .join(&tmp_base)?
            .join(tempdir_name)?;

        tempdir.create_dir_all()?;

        Ok(Self { path: tempdir })
    }

    pub fn path(&self) -> &VfsPath {
        &self.path
    }
}