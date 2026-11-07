use std::io::IsTerminal;
use std::path::Path;

use clap::Parser;
use log::{self, error, info};

use mamba::diagnostic::RenderedDiagnostic;
use mamba::io::{read_project, write_output};
use mamba::{compile, version, Artifact, Diagnostic, Options, Output, RenderStyle, Source};

use crate::cli::{Cli, ErrorFormat};

mod cli;

pub fn main() {
    #[cfg(windows)]
    let _ = ansi_term::enable_ansi_support();

    let cli_input = parse_cli();

    let colour = !cli_input.no_color
        && std::env::var_os("NO_COLOR").is_none()
        && std::io::stderr().is_terminal();

    loggerv::Logger::new()
        .verbosity(cli_input.v as u64)
        .level(cli_input.level)
        .line_numbers(cli_input.debug)
        .module_path(!cli_input.no_module_path)
        .colors(colour)
        .init()
        .unwrap();

    let target = cli_input.target.clone();
    let output = if cli_input.bin {
        Output::Object { target }
    } else if cli_input.asm {
        Output::Asm { target }
    } else {
        Output::Python
    };
    let options = Options::new(output).annotate(cli_input.annotate);

    info!("Mamba 🐍 {}", version());
    let current_dir = std::env::current_dir().unwrap_or_else(|err| {
        error!("Error while finding current directory: {err}");
        std::process::exit(1);
    });

    let style = if colour {
        RenderStyle::Ansi
    } else {
        RenderStyle::Plain
    };
    if let Err((diagnostics, sources)) = run(&current_dir, &cli_input, &options) {
        report(&diagnostics, &sources, style, cli_input.error_format);
        std::process::exit(1);
    }
}

/// Compile as `cli_input` asks, giving the diagnostics and the sources they are about on failure.
fn run(
    dir: &Path,
    cli_input: &Cli,
    options: &Options,
) -> Result<(), (Vec<Diagnostic>, Vec<Source>)> {
    let project = read_project(dir, cli_input.input.as_deref()).map_err(|errs| (errs, vec![]))?;
    let artifacts =
        compile(&project.sources, options).map_err(|errs| (errs, project.sources.clone()))?;

    if let Output::Asm { .. } = options.output {
        print_asm(&artifacts);
        return Ok(());
    }

    write_output(
        dir,
        cli_input.output.as_deref(),
        &project,
        artifacts,
        &options.output,
    )
    .map(|_| ())
    .map_err(|errs| (errs, vec![]))
}

/// The command line arguments, or exit.
///
/// Clap prints the error, or the help or version that was asked for, which exits successfully.
fn parse_cli() -> Cli {
    Cli::try_parse().unwrap_or_else(|err| {
        let _ = err.print();
        std::process::exit(if err.use_stderr() { 1 } else { 0 });
    })
}

/// Print each assembly artifact to stdout, since assembly is printed rather than written.
fn print_asm(artifacts: &[Artifact]) {
    for asm in artifacts.iter().filter_map(Artifact::as_text) {
        println!("{asm}");
    }
}

/// Print `diagnostics` to stderr, each with the text of the source it is about.
///
/// [compile] already gives each diagnostic only once.
fn report(diagnostics: &[Diagnostic], sources: &[Source], style: RenderStyle, format: ErrorFormat) {
    for diagnostic in diagnostics {
        let text = sources
            .iter()
            .find(|source| source.name == diagnostic.source)
            .map_or("", |source| source.text.as_str());

        match format {
            ErrorFormat::Human => eprintln!("{}", diagnostic.render(text, style)),
            ErrorFormat::Json => eprintln!("{}", to_json(diagnostic, text)),
        }
    }
}

/// `diagnostic` as one line of JSON, or why it could not be serialized.
fn to_json(diagnostic: &Diagnostic, text: &str) -> String {
    let rendered = RenderedDiagnostic::new(diagnostic, text);
    serde_json::to_string(&rendered).unwrap_or_else(|err| err.to_string())
}
