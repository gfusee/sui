use crate::errors::TempDirResult;
use std::env;
use std::path::PathBuf;
use uuid::Uuid;
use crate::wrappers::VirtualPath;

pub struct TempDir {
    path: VirtualPath
}

impl Drop for TempDir {
    fn drop(&mut self) {
        _ = self.path.remove_dir_all();
    }
}

impl TempDir {
    pub fn new(base: &VirtualPath) -> TempDirResult<Self> {
        let tempdir_name = format!("tmp-{}", Uuid::new_v4());

        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        let os_tempdir = env::temp_dir().canonicalize()?;

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        let tmp_base = PathBuf::from("tmp".to_string());

        let tempdir = base.root()
            .join(&os_tempdir)?
            .join(&tempdir_name)?;

        tempdir.create_dir_all()?;

        Ok(Self { path: tempdir })
    }

    pub fn path(&self) -> &VirtualPath {
        &self.path
    }
}