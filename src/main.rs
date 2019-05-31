#[macro_use]
extern crate clap;

use clap::App;
use leg::*;
use mamba::pipeline::mamba_to_python;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    head("mamba", Some("🐍"), Some(VERSION));

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(VERSION).get_matches();
    match std::env::current_dir() {
        Ok(current_dir) => {
            let in_path = matches.value_of("input");
            let out_path = matches.value_of("output");

            mamba_to_python(&current_dir, in_path, out_path);
        }
        e => error(format!("Error while finding current directory: {:#?}", e).as_str(), None, None)
    }
}
