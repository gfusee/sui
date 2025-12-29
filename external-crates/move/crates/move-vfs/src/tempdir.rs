use crate::errors::TempDirResult;
use crate::wrappers::VirtualPath;
use std::{env, mem};
use uuid::Uuid;
use vfs::VfsResult;

pub struct TempDir {
    path: Option<VirtualPath> // Always Some excepted after a call to close, useful to not leak memory
}

impl Drop for TempDir {
    fn drop(&mut self) {
        _ = self.path.as_ref().unwrap().remove_dir_all();
    }
}

impl TempDir {
    pub fn new(base: &VirtualPath) -> TempDirResult<Self> {
        let tempdir_name = format!("tmp-{}", Uuid::new_v4());

        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        let tmp_base = env::temp_dir().canonicalize()?;

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        let tmp_base = PathBuf::from("tmp".to_string());

        let tempdir = base.root()
            .join(&tmp_base)?
            .join(&tempdir_name)?;

        tempdir.create_dir_all()?;

        Ok(Self { path: Some(tempdir) })
    }

    pub fn path(&self) -> &VirtualPath {
        &self.path.as_ref().unwrap()
    }

    pub fn close(mut self) -> VfsResult<()> {
        self.path().remove_dir_all()?;

        // Avoids memory leaks
        self.path = None;

        mem::forget(self); // Drop should NOT be called after a call to close

        Ok(())
    }
}