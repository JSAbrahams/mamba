use clap::{ArgAction, Parser};

/// Transpile Mamba to Python code.
#[derive(Debug, Parser)]
#[command(
    name = "Mamba",
    author = "Joël Abrahams",
    about = "Transpile Mamba to Python code."
)]
pub struct Cli {
    /// Input file or directory.
    /// If file, file taken as input.
    /// If directory, recursively search all sub-directories for *.mamba files.
    /// If no input given, current directory used as input directory.
    #[arg(short = 'i', long = "input", value_name = "INPUT", value_parser)]
    pub input: Option<String>,

    /// Output directory to store Python files.
    /// Output directory structure reflects input directory structure.
    /// If no output given, 'target' directory created in current directory.
    #[arg(short = 'o', long = "output", value_name = "OUTPUT", value_parser)]
    pub output: Option<String>,

    /// Set level of verbosity:
    /// - `-v`   : info, error, warning printed to stderr (default)
    /// - `-vv`  : debug messages are printed
    /// - `-vvv` : trace messages are printed
    #[arg(short = 'v', action = ArgAction::Count)]
    pub v: u8,

    /// Add line numbers to log statements
    #[arg(short = 'd', long = "debug", action = ArgAction::SetTrue)]
    pub debug: bool,

    /// Disable the module path in the log statements
    #[arg(long = "no-module-path", action = ArgAction::SetTrue)]
    pub no_module_path: bool,

    /// Disable colorized output
    #[arg(long = "no-color", action = ArgAction::SetTrue)]
    pub no_color: bool,

    /// Print log level
    #[arg(short = 'l', long = "level", action = ArgAction::SetTrue)]
    pub level: bool,

    /// Enable type annotation of the output source.
    /// Currently still buggy feature.
    #[arg(short = 'a', long = "annotate", action = ArgAction::SetTrue)]
    pub annotate: bool,
}
