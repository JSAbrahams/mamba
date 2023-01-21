use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use glob::glob;
use pathdiff::diff_paths;

pub fn read_source(source_path: &Path) -> Result<String, String> {
    let mut source = String::new();
    OpenOptions::new()
        .read(true)
        .open(source_path)
        .map_err(|e| format!("{}: {}", e, source_path.display()))?
        .read_to_string(&mut source)
        .map_err(|e| format!("{}: {}", e, source_path.display()))?;
    Ok(source)
}

pub fn write_source(source: &str, out_path: &Path) -> Result<usize, String> {
    match out_path.parent() {
        Some(parent) => fs::create_dir_all(parent)
            .map_err(|e| format!("{}: {}", e, parent.display()))?,
        None => return Err(format!("No parent directory: {}", out_path.display()))
    };

    // LF instead of CRLF line endings
    let source = source.replace("\r\n", "\n");
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(out_path)
        .map_err(|e| format!("{}: {}", e, out_path.display()))?
        .write(source.as_ref())
        .map_err(|e| format!("{}: {}", e, out_path.display()))
}

/// Get all `*.mamba` files paths relative to given path.
///
/// If path is file, return file name.
/// If directory, return all `*.mamba` files as relative paths to given path.
pub fn relative_files(in_path: &Path) -> Result<Vec<OsString>, String> {
    if in_path.is_file() {
        let in_file_name = in_path.file_name().unwrap_or_else(|| unreachable!());
        return Ok(vec![in_file_name.to_os_string()]);
    }

    let pattern_path = in_path.to_owned().join("**").join("*.mamba");
    let pattern = pattern_path.as_os_str().to_string_lossy();
    let glob = glob(pattern.as_ref())
        .map_err(|e| format!("Unable to recursively find files: {e}"))?;

    let mut relative_paths = vec![];
    for absolute_result in glob {
        let absolute_path = absolute_result.map_err(|e| (e.to_string()))?;
        let relative_path = diff_paths(absolute_path.as_path(), in_path)
            .ok_or_else(|| String::from("Unable to create relative path"))?;
        relative_paths.push(relative_path.into_os_string());
    }

    Ok(relative_paths)
}
