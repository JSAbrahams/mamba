//! JavaScript bindings, with the `wasm` feature.
//!
//! Every function returns plain JavaScript objects, with camelCase fields.
//! A diagnostic has the same shape as a line of `mamba --error-format json`.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::diagnostic::RenderedDiagnostic;
use crate::{Diagnostic, Options, Output};

/// The options of [compile], as JavaScript passes them.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct JsOptions {
    /// `"python"`, the default, or `"asm"`.
    output: Option<String>,
    /// A target triple for `"asm"`, by default `x86_64-unknown-linux-gnu`.
    target: Option<String>,
    annotate: bool,
}

/// What [compile] and [tokens] return: the fields of `value`, or the diagnostics.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Outcome<'a, T: Serialize> {
    ok: bool,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    value: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<RenderedDiagnostic<'a>>>,
}

#[derive(Serialize)]
struct Text<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct Tokens<'a> {
    tokens: &'a [crate::Token],
}

/// Compile `source` to Python or assembly.
///
/// `options` is `{ output?: "python" | "asm", target?: string, annotate?: boolean }`, and may be left out.
/// Returns `{ ok: true, text }` or `{ ok: false, diagnostics }`.
/// Throws if the options are not of that shape.
#[wasm_bindgen]
pub fn compile(source: &str, options: JsValue) -> Result<JsValue, JsError> {
    let options = options_from_js(options)?;
    let compiled = crate::compile_str(source, &options);
    to_js(&outcome(&compiled, source, |text| Text { text }))
}

/// The tokens of `source`, for syntax highlighting.
///
/// Returns `{ ok: true, tokens }` or `{ ok: false, diagnostics }`.
#[wasm_bindgen]
pub fn tokens(source: &str) -> Result<JsValue, JsError> {
    let tokenized = crate::tokens(source);
    to_js(&outcome(&tokenized, source, |tokens| Tokens { tokens }))
}

fn outcome<'a, T, V: Serialize>(
    result: &'a Result<T, Vec<Diagnostic>>,
    source: &str,
    value: impl FnOnce(&'a T) -> V,
) -> Outcome<'a, V> {
    match result {
        Ok(ok) => Outcome {
            ok: true,
            value: Some(value(ok)),
            diagnostics: None,
        },
        Err(diagnostics) => Outcome {
            ok: false,
            value: None,
            diagnostics: Some(render_all(diagnostics, source)),
        },
    }
}

/// The version of Mamba.
#[wasm_bindgen]
pub fn version() -> String {
    String::from(crate::version())
}

fn options_from_js(options: JsValue) -> Result<Options, JsError> {
    let options: JsOptions = if options.is_undefined() || options.is_null() {
        JsOptions::default()
    } else {
        serde_wasm_bindgen::from_value(options).map_err(|err| JsError::new(&err.to_string()))?
    };

    let output = match options.output.as_deref() {
        None | Some("python") => Output::Python,
        Some("asm") => Output::Asm {
            target: options.target,
        },
        Some(other) => {
            let msg = format!("Unknown output '{other}': expected \"python\" or \"asm\"");
            return Err(JsError::new(&msg));
        }
    };
    Ok(Options::new(output).annotate(options.annotate))
}

fn render_all<'a>(diagnostics: &'a [Diagnostic], source: &str) -> Vec<RenderedDiagnostic<'a>> {
    diagnostics
        .iter()
        .map(|diagnostic| RenderedDiagnostic::new(diagnostic, source))
        .collect()
}

fn to_js(value: &impl Serialize) -> Result<JsValue, JsError> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|err| JsError::new(&err.to_string()))
}
