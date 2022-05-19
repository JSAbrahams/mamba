extern crate ansi_term;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate loggerv;


use clap::App;
use itertools::Itertools;

use mamba::transpile_directory;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() -> Result<(), String> {
    #[cfg(windows)]
        ansi_term::enable_ansi_support().unwrap();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).version(VERSION).get_matches();
    let in_path = matches.value_of("input");
    let out_path = matches.value_of("output");

    loggerv::Logger::new()
        .verbosity(matches.occurrences_of("v"))
        .level(matches.is_present("level"))
        .line_numbers(matches.is_present("debug"))
        .module_path(!matches.is_present("no-module-path"))
        .colors(!matches.is_present("no-color"))
        .init()
        .unwrap();

    info!("Mamba 🐍 {}", VERSION);
    let current_dir = std::env::current_dir().map_err(|err| {
        error!("Error while finding current directory: {}", err);
        format!("Error while finding current directory: {}", err)
    })?;

    transpile_directory(&current_dir, in_path, out_path)
        .map_err(|errors| {
            errors.iter().unique().for_each(|(ty, msg)| eprintln!("[error | {}] {}", ty, msg));
            match errors.first() {
                Some((ty, msg)) => format!(
                    "{} {} error occurred: {}",
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
