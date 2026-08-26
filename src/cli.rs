use clap::{ArgAction, Parser};

/// Transpile Mamba to Python code, compile it to a native binary, or print its assembly.
#[derive(Debug, Parser)]
#[command(
    name = "Mamba",
    author = "Joël Abrahams",
    about = "Transpile Mamba to Python code, compile it to a native binary, or print its assembly."
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
    /// Ignored with `--asm`, which always prints to stdout instead of writing a file.
    #[arg(short = 'o', long = "output", value_name = "OUTPUT", value_parser)]
    pub output: Option<String>,

    /// Output Python source (the default).
    #[arg(long = "python", action = ArgAction::SetTrue, conflicts_with_all = ["bin", "asm"])]
    pub python: bool,

    /// Compile and link a native executable via the Cranelift backend, instead of outputting
    /// Python source.
    /// Only a small subset of the language is currently supported: literals, arithmetic and
    /// comparison operators, if/else, top-level function definitions and calls, and `print`.
    #[arg(long = "bin", action = ArgAction::SetTrue, conflicts_with_all = ["python", "asm"])]
    pub bin: bool,

    /// Compile via the Cranelift backend and print the resulting disassembly to stdout, instead
    /// of outputting Python source or linking an executable. No file is written -- pipe stdout
    /// (e.g. `> out.s`) if you want to save it. Same language subset as `--bin` (see its help).
    /// Printed in AT&T syntax (`movq %rsp, %rbp`, source before destination) -- Cranelift's own
    /// disassembler doesn't support switching to Intel syntax.
    ///
    /// Only shows instructions, not the data section: a string literal (e.g. a `print("...")`
    /// argument) is compiled into the object's data section, not the instruction stream, so it
    /// won't appear in this output at all -- the instructions will only show it being loaded by
    /// an opaque symbol name (e.g. `load_ext_name userextname0+0, %rdi`).
    #[arg(long = "asm", action = ArgAction::SetTrue, conflicts_with_all = ["python", "bin"])]
    pub asm: bool,

    /// Target triple to pass to Cranelift, e.g. `x86_64-unknown-linux-gnu` (only meaningful with
    /// `--bin`/`--asm`; defaults to the host triple).
    #[arg(long = "target", value_name = "TARGET")]
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
