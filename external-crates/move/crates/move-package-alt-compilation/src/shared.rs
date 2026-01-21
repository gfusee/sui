// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{build_config::BuildConfig, layout::CompiledPackageLayout};
use move_vfs::VfsResult;
use move_vfs::wrappers::VirtualPath;

/// Computes the base directory for the build output based on the install_dir configuration.
/// If install_dir is specified, it will be used as the base (resolved relative to project_root if relative).
/// Otherwise, the project_root itself is used as the base.
pub fn get_install_base_path(
    project_root: &VirtualPath,
    build_config: &BuildConfig,
) -> VfsResult<VirtualPath> {
    if let Some(install_dir) = &build_config.install_dir {
        project_root.join(install_dir) // VirtualPath already handles relative vs absolute paths
    } else {
        Ok(project_root.clone())
    }
}

/// Computes the full build directory path, including the "build" subdirectory.
/// This is where compiled packages are actually stored.
pub fn get_build_output_path(
    project_root: &VirtualPath,
    build_config: &BuildConfig,
) -> VfsResult<VirtualPath> {
    let base_path = get_install_base_path(project_root, build_config)?;
    base_path.join(CompiledPackageLayout::Root.path())
}
