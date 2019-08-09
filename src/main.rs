#[macro_use]
extern crate clap;

use clap::App;
use leg::*;
use mamba::pipeline::transpile_directory;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() -> Result<(), String> {
    head("Mamba", Some("🐍"), Some(VERSION));
    let current_dir = std::env::current_dir().map_err(|err| {
        error(format!("Error while finding current directory: {:#?}", err).as_str(), None, None);
        format!("Error while finding current directory: {:#?}", err)
    })?;

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(VERSION).get_matches();
    let in_path = matches.value_of("input");
    let out_path = matches.value_of("output");

    transpile_directory(&current_dir, in_path, out_path)
        .map_err(|errors| {
            errors.iter().for_each(|(ty, msg)| error(msg, Some(ty), None));
            match errors.first() {
                Some((ty, msg)) => format!(
                    "{} {} type error occurred: {}",
                    match ty.chars().next() {
                        Some(c) if ['a', 'e', 'i', 'o', 'u'].contains(&c.to_ascii_lowercase()) =>
                            "An",
                        _ => "A"
                    },
                    ty,
                    msg
                ),
                None => String::new()
            }
        })
        .map(|_| ())
}
