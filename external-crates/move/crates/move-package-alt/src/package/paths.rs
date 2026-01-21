// Copyright (c) The Diem Core Contributors
//
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_vfs::{VfsError, VfsResult};
use serde::{Deserialize, de::DeserializeOwned};
use std::cmp::Ordering;
use std::{collections::BTreeMap, fmt::Debug};
use thiserror::Error;
use tracing::debug;

/// Lock file version written by this version of the compiler.  Backwards compatibility is
/// guaranteed (the compiler can read lock files with older versions), forward compatibility is not
/// (the compiler will fail to read lock files at newer versions).
///
/// V0: Base version.
/// V1: Adds toolchain versioning support.
/// V2: Adds support for managing addresses on package publish and upgrades.
/// V3: Renames dependency `name` field to `id` and adds a `name` field to store the name from the manifest.
/// V4: Package rewrite
const LOCKFILE_VERSION: usize = 4;

use super::{
    EnvironmentName,
    package_lock::{LockError, PackageSystemLock},
};
use crate::{
    compatibility::{
        legacy::LegacyEnvironment, legacy_lockfile::load_legacy_lockfile,
        legacy_parser::try_load_legacy_manifest,
    },
    errors::FileHandle,
    flavor::MoveFlavor,
    schema::{
        Environment, ParsedEphemeralPubs, ParsedLockfile, ParsedManifest, ParsedPublishedFile,
        RenderToml,
    },
};
use move_vfs::wrappers::VirtualPath;

/// A path to a directory containing a loaded Move package (in particular, the directory must have
/// a Move.toml)
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct PackagePath(OutputPath);

/// A path to a directory in which output can be generated. The directory must exist, but no files
/// are necessarily present
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputPath(VirtualPath);

/// A path to an ephemeral publication file that may be read or updated
#[derive(Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EphemeralPubfilePath(VirtualPath);

#[derive(Error, Debug)]
pub enum PackagePathError {
    #[error("Invalid directory at `{path}`")]
    InvalidDirectory { path: String },

    #[error("Package does not have a Move.toml file at `{path}`")]
    InvalidPackage { path: String },

    #[error("Path `{path}` does not refer to a file")]
    InvalidFile { path: String },

    #[error(transparent)]
    VfsError(#[from] VfsError),
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("error while parsing {file:?}: {source}")]
    TomlParseError {
        file: FileHandle,
        source: toml_edit::de::Error,
    },

    #[error("error while loading {file:?}: {source}")]
    IoError {
        file: String,
        source: std::io::Error,
    },

    #[error("Path `{path}` does not refer to a file")]
    InvalidFile { path: String },

    #[error("error while loading legacy manifest {file:?}: {source}")]
    LegacyError { file: String, source: anyhow::Error },

    #[error(transparent)]
    LockError(#[from] LockError),

    #[error(transparent)]
    VfsError(#[from] VfsError),

    #[error(
        "File {file:?} has version {version}, but this CLI only supports versions up to {max}; please upgrade your CLI"
    )]
    VersionError {
        file: String,
        version: usize,
        max: usize,
    },
}

pub type PackagePathResult<T> = Result<T, PackagePathError>;
pub type FileResult<T> = Result<T, FileError>;

/// Attempt to extract the name from the manifest file contained inside of `dir`.
/// Compatable with both modern and legacy files
pub fn read_name_from_manifest(dir: &VirtualPath) -> FileResult<String> {
    #[derive(Deserialize)]
    struct Header {
        name: String,
    }

    #[derive(Deserialize)]
    struct File {
        package: Header,
    }

    let path = dir.join("Move.toml")?;
    let f: File = parse_file(path.clone())?
        .ok_or(FileError::InvalidFile {
            path: path.as_str().to_string(),
        })?
        .1;
    Ok(f.package.name)
}

impl PackagePath {
    pub fn new(dir: VirtualPath) -> PackagePathResult<Self> {
        if !dir.is_dir()? {
            return Err(PackagePathError::InvalidDirectory {
                path: dir.as_str().to_string(),
            });
        }

        let result = Self(OutputPath(dir));

        let manifest_path = result.manifest_path()?;

        if !manifest_path.exists()? {
            return Err(PackagePathError::InvalidPackage {
                path: manifest_path.as_str().to_string(),
            });
        }

        Ok(result)
    }

    /// Acquire an exclusive lock for the files in this package
    pub(crate) fn lock(&self) -> FileResult<PackageSystemLock> {
        self.0.lock()
    }

    /// Parse and return the manifest file, failing if it doesn't exist or isn't correctly
    /// formatted
    pub(crate) fn read_manifest(
        &self,
        _mtx: &PackageSystemLock,
    ) -> FileResult<(FileHandle, ParsedManifest)> {
        let path = self.manifest_path()?;
        parse_file(path.clone())?.ok_or(FileError::InvalidFile {
            path: path.as_str().to_string(),
        })
    }

    /// Parse and return the lockfile if it exists, returning None if the file doesn't exist or if
    /// it is in a legacy format. Fails if the file exists and has a modern version number but
    /// isn't correctly formatted
    pub(crate) fn read_lockfile(
        &self,
        _mtx: &PackageSystemLock,
    ) -> FileResult<Option<(FileHandle, ParsedLockfile)>> {
        if !self.lockfile_path()?.exists()? {
            return Ok(None);
        }
        let lockfile_path = self.lockfile_path()?;
        let version = lockfile_version(lockfile_path.clone())?;
        if version < LOCKFILE_VERSION {
            Ok(None)
        } else if version == LOCKFILE_VERSION {
            parse_file(lockfile_path)
        } else {
            Err(FileError::VersionError {
                file: lockfile_path.as_str().to_string(),
                version,
                max: LOCKFILE_VERSION,
            })
        }
    }

    /// Returns any publications that can be extracted from a legacy lockfile. Returns None
    /// if the lockfile is not a legacy lockfile; returns an error if this is a legacy lockfile
    /// but it cannot be parsed
    pub(crate) fn read_legacy_lockfile(
        &self,
        _mtx: &PackageSystemLock,
    ) -> FileResult<Option<BTreeMap<EnvironmentName, LegacyEnvironment>>> {
        let path = self.lockfile_path()?;
        let pubs = load_legacy_lockfile(&path).map_err(|err| FileError::LegacyError {
            file: path.as_str().to_string(),
            source: err,
        })?;
        Ok(pubs)
    }

    /// Check whether this package contains a legacy manifest - returns `None` if it contains a
    /// non-legacy manifest, or an error if it contains an invalid legacy manifest file.
    pub(crate) fn read_legacy_manifest<F: MoveFlavor>(
        &self,
        default_env: &Environment,
        is_root: bool,
        _mtx: &PackageSystemLock,
    ) -> FileResult<Option<(FileHandle, ParsedManifest)>> {
        let path = self.manifest_path()?;
        try_load_legacy_manifest::<F>(self, default_env, is_root).map_err(|err| {
            FileError::LegacyError {
                file: path.as_str().to_string(),
                source: err,
            }
        })
    }

    /// Parse and return the pubfile if it exists, returning None if the file doesn't exist.
    /// Fails if the file exists and isn't correctly formatted
    pub(crate) fn read_pubfile<F: MoveFlavor>(
        &self,
        _mtx: &PackageSystemLock,
    ) -> FileResult<Option<(FileHandle, ParsedPublishedFile<F>)>> {
        parse_file(self.pubfile_path()?)
    }

    /// The path to the directory containing the package
    pub fn path(&self) -> &VirtualPath {
        self.0.path()
    }

    fn manifest_path(&self) -> VfsResult<VirtualPath> {
        self.0.manifest_path()
    }

    fn lockfile_path(&self) -> VfsResult<VirtualPath> {
        self.0.lockfile_path()
    }

    fn pubfile_path(&self) -> VfsResult<VirtualPath> {
        self.0.pubfile_path()
    }
}

impl PartialOrd for OutputPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.as_str().partial_cmp(other.0.as_str())
    }
}

impl Ord for OutputPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_str().cmp(other.0.as_str())
    }
}

impl OutputPath {
    /// Create a canonical path from the given [`dir`]. This function checks that there is a
    /// directory at `dir` and that it contains a valid Move package, i.e., it has a `Move.toml`
    /// file.
    pub fn new(dir: VirtualPath) -> PackagePathResult<Self> {
        if !dir.is_dir()? {
            Err(PackagePathError::InvalidDirectory {
                path: dir.as_str().to_string(),
            })
        } else {
            Ok(Self(dir))
        }
    }

    /// Acquire an exclusive lock for the files in this package
    pub(crate) fn lock(&self) -> FileResult<PackageSystemLock> {
        Ok(PackageSystemLock::new_for_project(self.path())?)
    }

    fn path(&self) -> &VirtualPath {
        &self.0
    }

    /// Replace the lockfile with the contents of `file`
    pub(crate) fn write_lockfile(
        &mut self,
        file: &ParsedLockfile,
        _mtx: &PackageSystemLock,
    ) -> FileResult<()> {
        render_file(&self.lockfile_path()?, file)
    }

    /// Replace the pubfile with the contents of `file`
    pub(crate) fn write_pubfile<F: MoveFlavor>(
        &mut self,
        file: &ParsedPublishedFile<F>,
        _mtx: &PackageSystemLock,
    ) -> FileResult<()> {
        render_file(&self.pubfile_path()?, file)
    }

    /// Read the contents of the lockfile from the output directory
    #[cfg(test)]
    pub async fn dump_lockfile(&self, mtx: &PackageSystemLock) -> ParsedLockfile {
        PackagePath(self.clone())
            .read_lockfile(mtx)
            .unwrap()
            .unwrap()
            .1
    }

    /// Read the contents of the pubfile from the output directory
    #[cfg(test)]
    pub async fn dump_pubfile<F: MoveFlavor>(
        &self,
        mtx: &PackageSystemLock,
    ) -> ParsedPublishedFile<F> {
        PackagePath(self.clone())
            .read_pubfile(mtx)
            .unwrap()
            .unwrap()
            .1
    }

    fn manifest_path(&self) -> VfsResult<VirtualPath> {
        self.path().join("Move.toml")
    }

    fn lockfile_path(&self) -> VfsResult<VirtualPath> {
        self.path().join("Move.lock")
    }

    fn pubfile_path(&self) -> VfsResult<VirtualPath> {
        self.path().join("Published.toml")
    }
}

impl EphemeralPubfilePath {
    /// Create `file`'s parent directory (thus ensuring that `file` can be created).
    pub fn new(file: VirtualPath) -> PackagePathResult<Self> {
        if let Err(e) = file.parent().create_dir_all() {
            debug!("unexpected error creating directory: {e:?}");
            Err(PackagePathError::InvalidFile {
                path: file.as_str().to_string(),
            })
        } else {
            Ok(Self(file))
        }
    }

    fn path(&self) -> &VirtualPath {
        &self.0
    }

    // TODO: we should require a lock for the pubfile, which we hold between reading and writing
    pub fn write_pubfile<F: MoveFlavor>(
        &mut self,
        file: &ParsedEphemeralPubs<F>,
    ) -> FileResult<()> {
        render_file(self.path(), file)
    }

    /// Parse the pubfile for the package; returning None if there is no file and an error if the
    /// file exists and can't be read or parsed
    pub fn read_pubfile<F: MoveFlavor>(
        &self,
    ) -> FileResult<Option<(FileHandle, ParsedEphemeralPubs<F>)>> {
        parse_file(self.path().clone())
    }
}

fn parse_file<T: DeserializeOwned>(path: VirtualPath) -> FileResult<Option<(FileHandle, T)>> {
    if !path.exists()? {
        Ok(None)
    } else if !path.is_file()? {
        Err(FileError::InvalidFile {
            path: path.as_str().to_string(),
        })
    } else {
        let file = FileHandle::new(path.clone()).map_err(|source| FileError::IoError {
            file: path.as_str().to_string(),
            source,
        })?;

        let parsed = toml_edit::de::from_str(file.source())
            .map_err(|source| FileError::TomlParseError { file, source })?;

        Ok(Some((file, parsed)))
    }
}

fn render_file<T: RenderToml>(path: &VirtualPath, value: &T) -> FileResult<()> {
    let rendered = value.render_as_toml();
    debug!("writing to {path:?}:\n{rendered}");
    write!(path.create_file()?, "{rendered}").map_err(|source| FileError::IoError {
        file: path.as_str().to_string(),
        source,
    })
}

/// Extract the version field from the lockfile (compatible with all formats)
fn lockfile_version(path: VirtualPath) -> FileResult<usize> {
    #[derive(Deserialize)]
    struct Header {
        #[serde(default)]
        version: usize,
    }

    #[derive(Deserialize)]
    struct Lockfile {
        #[serde(rename = "move")]
        header: Header,
    }

    let f: Lockfile = parse_file(path.clone())?
        .ok_or(FileError::InvalidFile {
            path: path.as_str().to_string(),
        })?
        .1;

    Ok(f.header.version)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use insta::assert_snapshot;

    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use path_clean::PathClean;
    use test_log::test;

    fn vpath(path_buf: PathBuf) -> VirtualPath {
        VirtualPath::physical()
            .unwrap()
            .cwd()
            .join(path_buf)
            .unwrap()
    }

    #[test]
    fn test_new() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create the directory structure: temp_dir/A/B/C
        let a = temp_dir.path().join("A");
        fs::create_dir_all(&a).unwrap();

        let b = a.join("B");
        fs::create_dir_all(&b).unwrap();
        let manifest_path = b.join("Move.toml");
        // Create a Move.toml file in the temporary directory
        fs::write(&manifest_path, "[package]\nname = 'test_package'\n").unwrap();

        let c = b.join("C");
        fs::create_dir_all(&c).unwrap();

        // Test going from C to B using relative path
        let relative_path = PathBuf::from("../../B");
        let package_path = PackagePath::new(vpath(c.join(relative_path))).unwrap();

        // The result should be the canonicalized path to B
        assert_eq!(package_path.0.0.as_str(), b.clean().to_string_lossy());
    }

    #[test]
    fn test_move_toml_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manifest_path = temp_dir.path().join("Move.toml");

        // Create a Move.toml file in the temporary directory
        fs::write(&manifest_path, "[package]\nname = 'test_package'\n").unwrap();

        assert!(PackagePath::new(vpath(manifest_path)).is_err());
    }

    /// It is important that the file containing the error is printed
    #[test(tokio::test)]
    async fn test_manifest_error() {
        let tempdir = tempfile::tempdir().unwrap();
        fs::write(
            tempdir.path().join("Move.toml"),
            indoc!(
                r###"
                [not-package]
                name = "fool"
                "###
            ),
        )
        .unwrap();

        let path = PackagePath::new(vpath(tempdir.path().to_path_buf())).unwrap();
        let mtx = path.lock().unwrap();
        let error = path.read_manifest(&mtx).unwrap_err().to_string();
        assert_snapshot!(error.replace(tempdir.path().to_string_lossy().as_ref(), "<TEMPDIR>"),
            @r###"
        error while parsing "<TEMPDIR>/Move.toml": TOML parse error at line 1, column 2
          |
        1 | [not-package]
          |  ^^^^^^^^^^^
        unknown field `not-package`, expected one of `package`, `environments`, `dependencies`, `dep-replacements`
        "###
        );
    }

    /// Parsing a lockfile from the future should fail with a message telling the user to upgrade
    #[test(tokio::test)]
    async fn test_future_lockfile() {
        let tempdir = tempfile::tempdir().unwrap();
        fs::write(tempdir.path().join("Move.toml"), "manifest file").unwrap();
        fs::write(
            tempdir.path().join("Move.lock"),
            indoc!(
                r###"
                [move]
                version = 5

                flying-cars = "yep"
                "###
            ),
        )
        .unwrap();

        let path = PackagePath::new(vpath(tempdir.path().to_path_buf())).unwrap();
        let mtx = path.lock().unwrap();
        let error = path.read_lockfile(&mtx).unwrap_err().to_string();
        assert_snapshot!(error.replace(tempdir.path().to_string_lossy().as_ref(), "<TEMPDIR>"),
            @r###"File "<TEMPDIR>/Move.lock" has version 5, but this CLI only supports versions up to 4; please upgrade your CLI"###
        );
    }
}
