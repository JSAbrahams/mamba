use clap::Parser;
use itertools::Itertools;
use log::{self, error, info};

use mamba::backend::Backend;
use mamba::cli::Cli;
use mamba::{transpile_dir, Arguments};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() -> Result<(), String> {
    #[cfg(windows)]
    ansi_term::enable_ansi_support().unwrap();

    // if error, then defer printing error to clap
    let Ok(cli_input) = Cli::try_parse().map_err(|e| e.print()) else {
        return Err(String::new());
    };

    loggerv::Logger::new()
        .verbosity(cli_input.v as u64)
        .level(cli_input.level)
        .line_numbers(cli_input.debug)
        .module_path(!cli_input.no_module_path)
        .colors(!cli_input.no_color)
        .init()
        .unwrap();

    let backend = if cli_input.bin {
        Backend::Bin {
            target: cli_input.target.clone(),
        }
    } else if cli_input.asm {
        Backend::Asm {
            target: cli_input.target.clone(),
        }
    } else {
        Backend::Python
    };

    let arguments = Arguments {
        annotate: cli_input.annotate,
        backend,
    };

    info!("Mamba 🐍 {VERSION}");
    let current_dir = std::env::current_dir().map_err(|err| {
        error!("Error while finding current directory: {err}");
        format!("Error while finding current directory: {err}")
    })?;

    transpile_dir(
        &current_dir,
        cli_input.input.as_deref(),
        cli_input.output.as_deref(),
        &arguments,
    )
    .map_err(|errors| {
        errors.iter().unique().for_each(|msg| eprintln!("{msg}"));
        match errors.first() {
            Some(msg) => msg.clone(),
            None => String::new(),
        }
    })
    .map(|_| ())
}
