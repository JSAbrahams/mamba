#[macro_use]
extern crate clap;

use clap::App;
use mamba::pipeline::mamba_to_python;
use std::path::PathBuf;

pub fn main() -> Result<(), String> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;

    let in_path = current_dir
        .join(matches.value_of("INPUT").ok_or("No input given").map(PathBuf::from)?);
    let out_path = matches.value_of("OUTPUT").map(PathBuf::from);

    match out_path {
        Some(out_path) =>
            mamba_to_python(&in_path, Some(current_dir.join(out_path.as_path()).as_path())),
        None => mamba_to_python(&in_path, None)
    }?;

    Ok(())
}
