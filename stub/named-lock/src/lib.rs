#![cfg_attr(docsrs, feature(doc_cfg))]

//! This crate provides a simple and cross-platform implementation of named locks.
//! You can use this to lock sections between processes.
//!
//! ## Example
//!
//! ```rust
//! use named_lock::NamedLock;
//! use named_lock::Result;
//!
//! fn main() -> Result<()> {
//!     let lock = NamedLock::create("foobar")?;
//!     let _guard = lock.lock()?;
//!
//!     // Do something...
//!
//!     Ok(())
//! }
//! ```

use once_cell::sync::Lazy;
use parking_lot::{Mutex, MutexGuard};
use std::collections::HashMap;
use std::marker::PhantomData;
#[cfg(unix)]
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

mod error;

pub use crate::error::*;

type NameType = String;

/// Cross-process lock that is identified by name.
#[derive(Debug)]
pub struct NamedLock {}

impl NamedLock {
    /// Create/open a named lock.
    ///
    /// # UNIX
    ///
    /// This will create/open a file and use [`flock`] on it. The path of
    /// the lock file will be `$TMPDIR/<name>.lock`, or `/tmp/<name>.lock`
    /// if `TMPDIR` environment variable is not set.
    ///
    /// If you want to specify the exact path, then use [NamedLock::with_path].
    ///
    /// # Windows
    ///
    /// This will create/open a [global] mutex with [`CreateMutexW`].
    ///
    ///
    /// [`flock`]: https://linux.die.net/man/2/flock
    /// [global]: https://docs.microsoft.com/en-us/windows/win32/termserv/kernel-object-namespaces
    /// [`CreateMutexW`]: https://docs.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw
    pub fn create(name: &str) -> Result<NamedLock> {
        Ok(NamedLock {})
    }

    /// Create/open a named lock on specified path.
    ///
    /// # Notes
    ///
    /// * This function does not append `.lock` on the path
    /// * Parent directories must exist
    #[cfg(unix)]
    #[cfg_attr(docsrs, doc(cfg(unix)))]
    pub fn with_path<P>(path: P) -> Result<NamedLock>
    where
        P: AsRef<Path>,
    {
        NamedLock::create(path.as_ref().to_owned())
    }

    /// Try to lock named lock.
    ///
    /// If it is already locked, `Error::WouldBlock` will be returned.
    pub fn try_lock(&self) -> Result<NamedLockGuard> {
        Ok(NamedLockGuard { _phantom: PhantomData })
    }

    /// Lock named lock.
    pub fn lock(&self) -> Result<NamedLockGuard> {
        Ok(NamedLockGuard { _phantom: PhantomData })
    }
}

/// Scoped guard that unlocks NamedLock.
#[derive(Debug)]
pub struct NamedLockGuard<'r> {
    _phantom: PhantomData<&'r ()>,
}