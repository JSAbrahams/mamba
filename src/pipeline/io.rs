use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub fn read_source(source_path: &PathBuf) -> Result<String, (String, String)> {
    let mut source = String::new();
    OpenOptions::new()
        .read(true)
        .open(source_path.clone())
        .map_err(|e| (String::from("input"), format!("{}: '{}'", e, source_path.display())))?
        .read_to_string(&mut source)
        .map_err(|e| (String::from("input"), format!("{}: '{}'", e, source_path.display())))?;

    Ok(source)
}

pub fn write_source(source: &str, out_path: &PathBuf) -> Result<usize, (String, String)> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(out_path.clone())
        .map_err(|e| (String::from("output"), format!("{}: '{}'", e, out_path.display())))?
        .write(source.as_ref())
        .map_err(|e| (String::from("output"), format!("{}: '{}'", e, out_path.display())))
}
