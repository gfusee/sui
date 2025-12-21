use fs4::fs_std::FileExt;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use thiserror::Error;
use tracing::debug;
use vfs::{VfsError, VfsResult};
use crate::git::get_cache_path;
use crate::logging::user_error;
use crate::vfs::wrappers::{Lock, Lockable, VirtualPath};

#[derive(Debug, Error)]
pub enum LockError {
    #[error(transparent)]
    VfsError(#[from] VfsError),

    #[error(
        "Unexpected error acquiring lock for package at {package} (lock file: `{lock}`): {source}"
    )]
    PackageLockError {
        package: String,
        lock: String,
        source: VfsError,
    },

    #[error("Unexpected error acquiring lock for {name} cache (path: `{path}`): {source}")]
    CacheLockError {
        name: String,
        path: String,
        source: VfsError,
    },
}

pub type LockResult<T> = Result<T, LockError>;

#[derive(Debug)]
pub struct PackageSystemLock {
    lockable: Lockable,
}

impl PackageSystemLock {
    /// Acquire a lock for doing git operations sequentially
    pub fn new_for_git(base: &VirtualPath, repo_id: &str) -> LockResult<Self> {
        let path = cache_path_for(base, repo_id)?;
        Self::new_for_path(&path, true).map_err(|source| LockError::CacheLockError {
            name: repo_id.to_string(),
            path: path.as_str().to_string(),
            source,
        })
    }

    /// Acquire a lock corresponding to the package contained in the directory `path`
    /// We do sequential operations per package (we acquire lock per package path).
    pub fn new_for_project(path: &VirtualPath) -> LockResult<Self> {
        let project_lock_path = cache_path_for(&path, digest_path(path).as_str())
            .expect("failed to get git cache folder lock");
        Self::new_for_path(&project_lock_path, true).map_err(|source| LockError::PackageLockError {
            package: path.as_str().to_string(),
            lock: project_lock_path.as_str().to_string(),
            source,
        })
    }

    fn new_for_path(path: &VirtualPath, should_truncate: bool) -> VfsResult<Self> {
        debug!("acquiring lock for {path:?}");
        let lock = path.open_and_lock_exclusive(should_truncate)?;
        lock.lock_exclusive()?;

        Ok(Self { lockable: lock })
    }
}

impl Drop for PackageSystemLock {
    fn drop(&mut self) {
        if let Err(err) = self.lockable.unlock() {
            user_error!(
                "Failed to release filesystem lock at {:?}: {err:?}",
                self.lockable
            );
        }
    }
}

fn cache_path_for(
    base: &VirtualPath,
    name: &str
) -> LockResult<VirtualPath> {
    let cache_path = get_cache_path(base);
    let project_lock_path = cache_path.join(format!(".{name}.lock"))?;

    // create dir if not exists.
    cache_path.create_dir_all().map_err(|source| LockError::CacheLockError {
        name: name.to_string(),
        path: project_lock_path.as_str().to_string(),
        source,
    })?;

    Ok(project_lock_path)
}

fn digest_path(path: &VirtualPath) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_str().as_bytes());
    let result = hasher.finalize();
    // Return hex representation
    format!("{:x}", result)
}
