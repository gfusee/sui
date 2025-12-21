// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use std::process::ExitStatus;

use crate::package::package_lock::LockError;
use thiserror::Error;
use tokio::process::Command;
use vfs::VfsError;
use crate::vfs::errors::TempDirError;
use crate::vfs::wrappers::VirtualPath;

pub type GitResult<T> = std::result::Result<T, GitError>;

#[derive(Error, Debug)]
pub enum GitError {
    #[error(
        "{repo} repo is dirty. Please clean it up before proceeding or pass the `--allow-dirty` flag"
    )]
    Dirty { repo: String },

    #[error("could not extract a sha for repo {repo} and rev {rev}")]
    NoSha { repo: String, rev: String },

    #[error("given a short sha for repo {repo} and rev {rev}, but no git cache path")]
    NoCachePath { repo: String, rev: String },

    #[error("error while executing git command `{cmd}`{cwd_note}:\n{kind:?}")]
    CommandError {
        cmd: String,
        cwd_note: String,
        #[source]
        kind: CommandErrorKind,
    },

    #[error("Error while creating temporary directory: {0}")]
    TempDirectory(std::io::Error),

    #[error("relative path `{path}` is not contained in the repository")]
    BadPath { path: String },

    #[error(transparent)]
    LockingError(#[from] LockError),

    #[error(transparent)]
    VfsError(#[from] VfsError),

    #[error(transparent)]
    TempDirError(#[from] TempDirError),
}

#[derive(Error, Debug)]
pub enum CommandErrorKind {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("command returned non-zero exit code {0}")]
    ErrorCode(ExitStatus),

    #[error("command produced non-utf8 output")]
    NonUtf8,
}

impl GitError {
    pub fn dirty(repo: &str) -> Self {
        Self::Dirty {
            repo: repo.to_string(),
        }
    }

    pub fn io_error(cmd: &Command, cwd: &Option<&VirtualPath>, error: std::io::Error) -> Self {
        Self::command_error(cmd, cwd, CommandErrorKind::IoError(error))
    }

    pub fn nonzero_exit_status(cmd: &Command, cwd: &Option<&VirtualPath>, code: ExitStatus) -> Self {
        Self::command_error(cmd, cwd, CommandErrorKind::ErrorCode(code))
    }

    pub fn non_utf_output(cmd: &Command, cwd: &Option<&VirtualPath>) -> Self {
        Self::command_error(cmd, cwd, CommandErrorKind::NonUtf8)
    }

    fn command_error(cmd: &Command, cwd: &Option<&VirtualPath>, kind: CommandErrorKind) -> Self {
        Self::CommandError {
            cmd: format!("{cmd:?}"),
            cwd_note: match cwd {
                Some(p) => format!(" (in directory `{p:?}`)"),
                None => String::new(),
            },
            kind,
        }
    }

    /// Construct an error for the case when we can't find a sha for revision `rev` in `repo`
    pub fn no_sha(repo_url: &str, rev: &str) -> Self {
        Self::NoSha {
            repo: repo_url.to_string(),
            rev: rev.to_string(),
        }
    }

    /// Construct an error for the case when no git cache path was provided
    /// for revision `rev` in `repo`, where `rev` is a short sha
    pub fn no_cache_path(repo_url: &str, rev: &str) -> Self {
        Self::NoCachePath {
            repo: repo_url.to_string(),
            rev: rev.to_string(),
        }
    }
}
