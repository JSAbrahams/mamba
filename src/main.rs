#[macro_use]
extern crate clap;

use clap::App;
use leg::*;
use mamba::pipeline::mamba_to_python;
use std::path::PathBuf;

pub fn main() -> Result<(), ()> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let current_dir = std::env::current_dir().map_err(|e| {
        error(format!("Error while finding current directory: {:#?}", e).as_str(), None, None)
    })?;

    let in_path = current_dir.join(
        matches
            .value_of("input")
            .ok_or("No input given")
            .map_err(|e| error(e, None, None))
            .map(PathBuf::from)?
    );
    let out_path = matches.value_of("output").map(PathBuf::from);

    match out_path {
        Some(out_path) =>
            mamba_to_python(&in_path, Some(current_dir.join(out_path.as_path()).as_path())),
        None => mamba_to_python(&in_path, None)
    }
    .map_err(|e| error(e.to_string().as_str(), None, None))?;
    Ok(())
}
