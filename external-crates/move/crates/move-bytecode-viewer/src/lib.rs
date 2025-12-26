// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    bytecode_viewer::BytecodeViewer, source_viewer::ModuleViewer,
    tui::tui_interface::start_tui_with_interface, viewer::Viewer,
};
use clap::Parser;
use move_binary_format::file_format::CompiledModule;
use move_bytecode_source_map::{source_map::SourceMap, utils::source_map_from_file};
use move_package_alt_vfs::wrappers::VirtualPath;
use std::{
    fs,
    path::PathBuf,
};

pub mod bytecode_viewer;
pub mod interfaces;
pub mod source_viewer;
pub mod tui;
pub mod viewer;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct BytecodeViewerConfig {
    /// The path to the module binary
    #[clap(long = "module-path", short = 'b')]
    pub module_binary_path: PathBuf,

    /// The path to the source map for the module binary
    #[clap(long = "source-map-path")]
    pub module_sourcemap_path: PathBuf,

    /// The path to the source file
    #[clap(long = "source-path", short = 's')]
    pub source_file_path: PathBuf,
}

pub fn start_viewer_in_memory(
    compiled_module: CompiledModule,
    source_map: SourceMap,
    source_path: &VirtualPath,
) {
    let module_viewer = ModuleViewer::new(compiled_module.clone(), source_map.clone(), source_path);
    let bytecode_viewer = BytecodeViewer::new(source_map, &compiled_module);
    let interface = Viewer::new(module_viewer, bytecode_viewer);
    start_tui_with_interface(interface).unwrap();
}

impl BytecodeViewerConfig {
    pub fn start_viewer(&self) {
        let bytecode_bytes =
            fs::read(&self.module_binary_path).expect("Unable to read bytecode file");
        let compiled_module = CompiledModule::deserialize_with_defaults(&bytecode_bytes)
            .expect("Module blob can't be deserialized");

        let root_path = VirtualPath::physical()
            .expect("Physical filesystem should be available");

        let module_sourcemap_path = root_path
            .join(&self.module_sourcemap_path)
            .expect(&format!("Path should be valid {}", self.module_binary_path.to_string_lossy()));

        let source_map = source_map_from_file(&module_sourcemap_path)
            .expect("Unable to read source map from file");

        let source_path = root_path
            .join(&self.source_file_path)
            .expect(&format!("Source file path should be valid {}", self.source_file_path.to_string_lossy()));

        start_viewer_in_memory(compiled_module, source_map, &source_path)
    }
}
