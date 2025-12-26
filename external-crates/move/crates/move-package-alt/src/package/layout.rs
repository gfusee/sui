// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use move_package_alt_vfs::wrappers::VirtualPath;

/// References file for documentation generation
pub const REFERENCE_TEMPLATE_FILENAME: &str = "references.md";

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SourcePackageLayout {
    Sources,
    Specifications,
    Tests,
    Scripts,
    Examples,
    Manifest,
    Lock,
    DocTemplates,
}

impl SourcePackageLayout {
    /// A Move source package is laid out on-disk as
    /// a_move_package
    /// ├── Move.toml      (required)
    /// ├── Move.lock      (optional)
    /// ├── sources        (required)
    /// ├── examples       (optional, dev mode)
    /// ├── scripts        (optional)
    /// ├── specifications (optional)
    /// ├── doc_templates      (optional)
    /// └── tests          (optional, test mode)
    pub fn path(&self) -> &str {
        self.location_str()
    }

    pub fn try_find_root(starting_path: VirtualPath) -> Result<VirtualPath> {
        let mut current_path = starting_path.clone();
        loop {
            if current_path.join(Self::Manifest.path())?.is_file()? {
                break Ok(current_path);
            }
            if !current_path.pop()? {
                bail!(
                    "Unable to find package manifest at '{}/{}' or in its parents",
                    starting_path.as_str(),
                    Self::Manifest.path().to_string()
                )
            }
        }
    }

    pub fn location_str(&self) -> &'static str {
        match self {
            Self::Sources => "sources",
            Self::Manifest => "Move.toml",
            Self::Lock => "Move.lock",
            Self::Tests => "tests",
            Self::Scripts => "scripts",
            Self::Examples => "examples",
            Self::Specifications => "specifications",
            Self::DocTemplates => "doc_templates",
        }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            Self::Sources | Self::Manifest => false,
            Self::Tests
            | Self::Scripts
            | Self::Examples
            | Self::Specifications
            | Self::DocTemplates
            | Self::Lock => true,
        }
    }
}
