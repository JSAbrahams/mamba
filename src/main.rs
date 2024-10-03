extern crate ansi_term;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate loggerv;

use clap::App;
use itertools::Itertools;

use mamba::{transpile_dir, Arguments};

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

    let arguments = Arguments { annotate: matches.is_present("annotate") };

    info!("Mamba 🐍 {}", VERSION);
    let current_dir = std::env::current_dir().map_err(|err| {
        error!("Error while finding current directory: {err}");
        format!("Error while finding current directory: {err}")
    })?;

    transpile_dir(&current_dir, in_path, out_path, &arguments)
        .map_err(|errors| {
            errors.iter().unique().for_each(|msg| eprintln!("{msg}"));
            match errors.first() {
                Some(msg) => msg.clone(),
                None => String::new(),
            }
        })
        .map(|_| ())
}
