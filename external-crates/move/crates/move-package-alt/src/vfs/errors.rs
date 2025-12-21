use thiserror::Error;
use vfs::VfsError;

pub type TempDirResult<T> = Result<T, TempDirError>;

#[derive(Error, Debug)]
pub enum TempDirError {
    #[error(transparent)]
    VfsError(#[from] VfsError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}