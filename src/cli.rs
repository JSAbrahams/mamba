use clap::{ArgAction, Parser};

/// Transpile Mamba to Python code, or compile it to a native binary.
#[derive(Debug, Parser)]
#[command(
    name = "Mamba",
    author = "Joël Abrahams",
    about = "Transpile Mamba to Python code, or compile it to a native binary."
)]
pub struct Cli {
    /// Input file or directory.
    /// If file, file taken as input.
    /// If directory, recursively search all sub-directories for *.mamba files.
    /// If no input given, current directory used as input directory.
    #[arg(short = 'i', long = "input", value_name = "INPUT", value_parser)]
    pub input: Option<String>,

    /// Output location.
    /// With `--python` (the default): output directory to store Python files, structured to
    /// reflect the input directory; if not given, a 'target' directory is created in the current
    /// directory.
    /// With `--bin`: path of the linked executable to produce; if not given, 'a.out' is created
    /// in the current directory.
    #[arg(short = 'o', long = "output", value_name = "OUTPUT", value_parser)]
    pub output: Option<String>,

    /// Output Python source (the default).
    #[arg(long = "python", action = ArgAction::SetTrue, conflicts_with = "bin")]
    pub python: bool,

    /// Compile and link a native executable via the Cranelift backend, instead of outputting
    /// Python source.
    /// Only a small subset of the language is currently supported: literals, arithmetic and
    /// comparison operators, if/else, top-level function definitions and calls, and `print`.
    #[arg(long = "bin", action = ArgAction::SetTrue, conflicts_with = "python")]
    pub bin: bool,

    /// Target triple to pass to Cranelift, e.g. `x86_64-unknown-linux-gnu` (only meaningful with
    /// `--bin`; defaults to the host triple).
    #[arg(long = "target", value_name = "TARGET", requires = "bin")]
    pub target: Option<String>,

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
