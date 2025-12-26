// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_package_alt_vfs::wrappers::VirtualPath;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum CompiledPackageLayout {
    BuildInfo,
    Root,
    Dependencies,
    Sources,
    DebugInfo,
    LockFiles,
    CompiledModules,
    CompiledDocs,
    Disassembly,
}

impl CompiledPackageLayout {
    pub fn path(&self) -> &str {
        match self {
            Self::BuildInfo => "BuildInfo.yaml",
            Self::Root => "build",
            Self::Dependencies => "dependencies",
            Self::Sources => "sources",
            Self::DebugInfo => "debug_info",
            Self::LockFiles => "locks",
            Self::CompiledModules => "bytecode_modules",
            Self::CompiledDocs => "docs",
            Self::Disassembly => "disassembly",
        }
    }

    pub fn path_to_file_after_category(virtual_path: &VirtualPath) -> PathBuf {
        let path = PathBuf::from(virtual_path.as_str());
        let mut suffix_components = vec![];
        // reverse iterate until Root is found
        for component in path.components().rev() {
            let component_path: &Path = component.as_ref();
            if component_path == Self::Root.path() {
                break;
            }
            suffix_components.push(component);
        }
        // pop root package name
        suffix_components.pop();
        // pop category
        suffix_components.pop();
        // put the components back in order
        suffix_components.reverse();
        // make the path
        suffix_components.into_iter().collect()
    }
}
