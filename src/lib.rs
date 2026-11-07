//! Mamba: a statically typed, Python-like language, and its compiler.
//!
//! The crate has no default features.
//! As it is, it is the pure compiler: [compile], [compile_str], [version] and [Diagnostic].
//! It reads no files and writes none, so it runs anywhere, WebAssembly included.
//!
//! - `native` adds [io]: compiling directories and linking executables.
//! - `cli` builds the `mamba` binary.
//! - `wasm` adds the JavaScript bindings.
//! - `serde` makes diagnostics serializable.
//!
//! ```
//! use mamba::{compile_str, Options};
//!
//! let python = compile_str("print(\"hello\")", &Options::default()).unwrap();
//! assert!(python.contains("print"));
//! ```

use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

use log::trace;

use crate::backend::python::GenArguments;
use crate::check::ast::ASTTy;
use crate::check::check;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::AST;

pub use crate::diagnostic::{Diagnostic, Label, LineCol, RenderStyle, Severity, Span, Stage};

pub mod common;

pub mod backend;
pub mod check;
pub mod parse;

pub mod diagnostic;

#[cfg(feature = "native")]
pub mod io;
#[cfg(feature = "wasm")]
pub mod wasm;

/// One Mamba source: its text, and optionally a name to show in diagnostics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Source {
    pub name: Option<PathBuf>,
    pub text: String,
}

/// What to compile Mamba to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Output {
    /// Python source, the most complete backend.
    #[default]
    Python,
    /// Assembly text from the machine code backend, which supports a small subset of the language.
    ///
    /// In AT&T syntax, and only the instructions: string literals live in the data section, which is not shown.
    /// `target` is a target triple such as `x86_64-unknown-linux-gnu`.
    /// If it is [None], it is the host with the `native` feature, and `x86_64-unknown-linux-gnu` without it.
    Asm { target: Option<String> },
    /// An object file from the machine code backend, which the `native` feature can link into an executable.
    ///
    /// `target` is as for [Output::Asm].
    Object { target: Option<String> },
}

/// How to compile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    pub output: Output,
    /// Add type annotations to the generated Python.
    pub annotate: bool,
}

/// What compiling one source produces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Artifact {
    /// Python or assembly.
    Text(String),
    /// An object file.
    Bytes(Vec<u8>),
}

impl Source {
    /// A source without a name.
    pub fn new(text: impl Into<String>) -> Source {
        Source {
            name: None,
            text: text.into(),
        }
    }

    /// A source named `name`, which diagnostics show.
    pub fn named(name: impl Into<PathBuf>, text: impl Into<String>) -> Source {
        Source {
            name: Some(name.into()),
            text: text.into(),
        }
    }

    fn name(&self) -> Option<&Path> {
        self.name.as_deref()
    }
}

impl Options {
    pub fn new(output: Output) -> Options {
        Options {
            output,
            annotate: false,
        }
    }

    #[must_use]
    pub fn annotate(self, annotate: bool) -> Options {
        Options { annotate, ..self }
    }
}

impl Artifact {
    /// The text of Python or assembly, or [None] for an object file.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Artifact::Text(text) => Some(text),
            Artifact::Bytes(_) => None,
        }
    }

    /// The bytes of the artifact: an object file, or the text as UTF-8.
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Artifact::Text(text) => text.into_bytes(),
            Artifact::Bytes(bytes) => bytes,
        }
    }
}

/// The version of Mamba: what `git describe --tags` gave when it was built, such as `v0.3.6-12-g3f2a1bc`.
///
/// Built without git or without tags, as from crates.io, it is the version in Cargo.toml, such as `0.3.6`.
pub fn version() -> &'static str {
    env!("MAMBA_VERSION")
}

/// Compile Mamba sources, giving one artifact per source, in order.
///
/// The sources are checked together, so that one may use what another defines.
/// On failure, every diagnostic found is returned, not just the first.
pub fn compile(sources: &[Source], options: &Options) -> Result<Vec<Artifact>, Vec<Diagnostic>> {
    let (ctx, typed) = check_all(sources)?;

    let results = typed
        .iter()
        .zip(sources)
        .map(|(ast_ty, source)| generate(ast_ty, source, options, &ctx));
    let (artifacts, errors): (Vec<_>, Vec<_>) = results.partition(Result::is_ok);
    if !errors.is_empty() {
        return Err(unique(errors.into_iter().map(Result::unwrap_err)));
    }

    trace!("Generated {} artifacts", artifacts.len());
    Ok(artifacts.into_iter().map(Result::unwrap).collect())
}

/// Compile one unnamed source to text: Python or assembly.
///
/// Asking for an [Output::Object] gives a diagnostic, as an object file is not text: use [compile] for that.
pub fn compile_str(text: &str, options: &Options) -> Result<String, Vec<Diagnostic>> {
    if let Output::Object { .. } = options.output {
        let msg = "An object file is not text: use `compile` to get its bytes";
        return Err(vec![Diagnostic::error(Stage::Generate, msg)]);
    }

    let mut artifacts = compile(&[Source::new(text)], options)?;
    match artifacts.pop() {
        Some(Artifact::Text(text)) => Ok(text),
        _ => unreachable!("one text source gives one text artifact"),
    }
}

/// Parse and type check `sources`, which every output needs.
fn check_all(sources: &[Source]) -> Result<(Context, Vec<ASTTy>), Vec<Diagnostic>> {
    let parsed = sources.iter().map(parse_source);
    let (asts, errors): (Vec<_>, Vec<_>) = parsed.partition(Result::is_ok);
    if !errors.is_empty() {
        return Err(unique(errors.into_iter().map(Result::unwrap_err)));
    }
    let asts: Vec<AST> = asts.into_iter().map(Result::unwrap).collect();
    trace!("Parsed {} files", asts.len());

    let ctx = build_context(&asts, sources)?;

    let checked = asts
        .iter()
        .zip(sources)
        .map(|(ast, source)| check_source(ast, source, &ctx));
    let (typed, errors): (Vec<_>, Vec<_>) = checked.partition(Result::is_ok);
    if !errors.is_empty() {
        return Err(unique(errors.into_iter().flat_map(Result::unwrap_err)));
    }

    trace!("Checked {} files", typed.len());
    Ok((ctx, typed.into_iter().map(Result::unwrap).collect()))
}

/// Parse one source.
///
/// The parser lexes too, and reports a lex error as a parse error.
fn parse_source(source: &Source) -> Result<AST, Diagnostic> {
    let (text, name) = (source.text.as_str(), source.name());
    text.parse::<AST>()
        .map_err(|err| Diagnostic::from_parse(&err, text, name))
}

/// Build the context that every source is checked in.
///
/// Its errors do not record which source they are in, so they name one only when there is just one.
fn build_context(asts: &[AST], sources: &[Source]) -> Result<Context, Vec<Diagnostic>> {
    let only = match sources {
        [source] => Some(source),
        _ => None,
    };
    let text = only.map(|source| source.text.as_str());
    let name = only.and_then(Source::name);

    Context::try_from(asts).map_err(|errs| {
        unique(
            errs.iter()
                .map(|err| Diagnostic::from_type(err, text, name)),
        )
    })
}

/// Type check one parsed source in `ctx`.
fn check_source(ast: &AST, source: &Source, ctx: &Context) -> Result<ASTTy, Vec<Diagnostic>> {
    let text = Some(source.text.as_str());
    check(ast, ctx).map_err(|errs| {
        errs.iter()
            .map(|err| Diagnostic::from_type(err, text, source.name()))
            .collect()
    })
}

/// Generate what `options` asks for from one checked source.
fn generate(
    ast_ty: &ASTTy,
    source: &Source,
    options: &Options,
    ctx: &Context,
) -> Result<Artifact, Diagnostic> {
    let (text, name) = (source.text.as_str(), source.name());
    match &options.output {
        Output::Python => {
            let gen_args = GenArguments {
                annotate: options.annotate,
            };
            backend::python::gen_arguments(ast_ty, &gen_args, ctx)
                .map(|core| Artifact::Text(format!("{core}")))
                .map_err(|err| Diagnostic::from_unimplemented(&err, text, name))
        }
        Output::Asm { target } => backend::cranelift::disassemble(ast_ty, target.as_deref(), ctx)
            .map(Artifact::Text)
            .map_err(|err| Diagnostic::from_backend(&err, text, name)),
        Output::Object { target } => backend::cranelift::compile(ast_ty, target.as_deref(), ctx)
            .map(Artifact::Bytes)
            .map_err(|err| Diagnostic::from_backend(&err, text, name)),
    }
}

/// The diagnostics in order, each only once.
fn unique(diagnostics: impl IntoIterator<Item = Diagnostic>) -> Vec<Diagnostic> {
    let mut seen = HashSet::new();
    diagnostics
        .into_iter()
        .filter(|diagnostic| seen.insert(diagnostic.clone()))
        .collect()
}
