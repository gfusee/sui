use std::path::Path;
use std::{fs, io};
use wasi::filesystem::types::{DescriptorFlags, OpenFlags, PathFlags};

pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let preopens = wasi::filesystem::preopens::get_directories();
    let root = preopens.into_iter().next().unwrap().0;

    let path_str = path
        .as_ref()
        .to_str()
        .ok_or_else(|| io::Error::other("Cannot convert path to str"))?;

    // Node.js wasip2 shim doesn't allow to read absolute paths.
    // As we are reading from root, we can remove "/"
    let path_str = if path_str.starts_with("/") {
        path_str.strip_prefix("/").unwrap()
    } else {
        path_str
    };

    let Ok(file) = root.open_at(
        PathFlags::empty(),
        path_str,
        OpenFlags::empty(),
        DescriptorFlags::READ,
    ) else {
        return Err(io::Error::other("failed to open file"));
    };

    let mut offset = 0;
    let mut out = Vec::new();

    loop {
        // Read 4KiB
        let (bytes, ended) = file.read(4096, offset).expect("read failed");

        if !bytes.is_empty() {
            out.extend_from_slice(&bytes);
            offset += bytes.len() as u64;
        }

        if ended || bytes.is_empty() {
            break;
        }
    }

    Ok(String::from_utf8(out).unwrap())
}
