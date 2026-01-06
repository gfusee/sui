// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::source_map::SourceMap;
use anyhow::{Result, format_err};
use move_ir_types::location::Loc;
use move_vfs::wrappers::VirtualPath;
use std::io::{Read, Write};

pub type Error = (Loc, String);
pub type Errors = Vec<Error>;

pub fn source_map_from_file(file_path: &VirtualPath) -> Result<SourceMap> {
    if file_path.extension().is_some_and(|ext| ext == "json") {
        return deserialize_from_json(file_path);
    }

    let mut bytes = Vec::new();
    file_path.open_file()?.read(&mut bytes)?;
    bcs::from_bytes::<SourceMap>(&bytes)
        .map_err(|_| format_err!("Error deserializing into source map"))
}

pub fn serialize_to_json_string(map: &SourceMap) -> Result<String> {
    serde_json::to_string_pretty(map).map_err(|e| format_err!("Error serializing to json: {}", e))
}

pub fn serialize_to_json(map: &SourceMap) -> Result<Vec<u8>> {
    serde_json::to_vec(map).map_err(|e| format_err!("Error serializing to json: {}", e))
}

pub fn serialize_to_json_file(map: &SourceMap, file_path: &VirtualPath) -> Result<()> {
    let json = serialize_to_json_string(map)?;
    write!(file_path.create_file()?, "{}", json)?;
    Ok(())
}

pub fn deserialize_from_json(file_path: &VirtualPath) -> Result<SourceMap> {
    let json = file_path.read_to_string()?;
    serde_json::from_str(&json).map_err(|e| format_err!("Error deserializing from json: {}", e))
}

pub fn convert_to_json(file_path: &VirtualPath) -> Result<()> {
    let map = source_map_from_file(file_path)?;
    let json_file_path = file_path.with_extension("json")?;
    serialize_to_json_file(&map, &json_file_path)
}
