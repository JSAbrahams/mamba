use std::io::IsTerminal;
use std::path::Path;

use clap::Parser;
use itertools::Itertools;
use log::{self, error, info};

use mamba::diagnostic::RenderedDiagnostic;
use mamba::io::{read_project, write_output};
use mamba::{compile, version, Artifact, Diagnostic, Options, Output, RenderStyle, Source};

use crate::cli::{Cli, ErrorFormat};

mod cli;

pub fn main() {
    #[cfg(windows)]
    let _ = ansi_term::enable_ansi_support();

    // if error, then defer printing error to clap
    let Ok(cli_input) = Cli::try_parse().map_err(|e| e.print()) else {
        std::process::exit(1);
    };

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

    // Assembly is printed, not written.
    if let Output::Asm { .. } = options.output {
        for artifact in &artifacts {
            if let Artifact::Text(asm) = artifact {
                println!("{asm}");
            }
        }
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

/// Print `diagnostics` to stderr, each with the text of the source it is about.
fn report(diagnostics: &[Diagnostic], sources: &[Source], style: RenderStyle, format: ErrorFormat) {
    for diagnostic in diagnostics.iter().unique() {
        let text = sources
            .iter()
            .find(|source| source.name == diagnostic.source)
            .map_or("", |source| source.text.as_str());

        match format {
            ErrorFormat::Human => eprintln!("{}", diagnostic.render(text, style)),
            ErrorFormat::Json => {
                let rendered = RenderedDiagnostic::new(diagnostic, text);
                match serde_json::to_string(&rendered) {
                    Ok(json) => eprintln!("{json}"),
                    Err(err) => eprintln!("{err}"),
                }
            }
        }
    }
}
