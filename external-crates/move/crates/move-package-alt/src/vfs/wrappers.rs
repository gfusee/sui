use std::cmp::Ordering;
use vfs::VfsPath;

/// A wrapper around VfsPath with PartialOrd and Ord
#[derive(PartialEq, Eq)]
pub(crate) struct OrdVfsPath(VfsPath);

impl OrdVfsPath {
    pub(crate) fn new(path: VfsPath) -> Self {
        Self(path)
    }

    pub fn as_path(&self) -> &VfsPath {
        &self.0
    }
}

impl PartialOrd for OrdVfsPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.as_str().partial_cmp(other.0.as_str())
    }
}

impl Ord for OrdVfsPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_str().cmp(other.0.as_str())
    }
}