// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod build;
pub mod coverage;
pub mod decompile;
pub mod disassemble;
pub mod docgen;
pub mod migrate;
pub mod new;
pub mod profile;
pub mod summary;
pub mod test;
pub mod update_deps;

use move_package_alt::package::layout::SourcePackageLayout;
use move_package_alt_vfs::wrappers::VirtualPath;
use std::path::Path;

/// Reroot the path if none is given, and convert it to a physical VirtualPath.
pub fn reroot_path(path: Option<&Path>) -> anyhow::Result<VirtualPath> {
    let virtual_cwd = VirtualPath::physical()?.cwd();

    let path = match path {
        Some(path) => virtual_cwd.join(path)?,
        None => virtual_cwd,
    };
    // Always root ourselves to the package root, and then compile relative to that.
    let rooted_path = SourcePackageLayout::try_find_root(path.clone())?;

    Ok(path.with_current_dir(rooted_path))
}
