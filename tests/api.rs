//! The public API of the crate as a library: compiling text, tokens, and diagnostics as data.
//!
//! Nothing here needs a feature, except the JSON round trip, which needs `serde`.

use std::fs;
use std::path::PathBuf;

use mamba::{
    compile, compile_str, tokens, version, Artifact, Diagnostic, LineCol, Options, Output, Source,
    Stage, TokenKind,
};
use test_case::test_case;
use tests_util::resource_path;

/// The text of the fixture `name` of `tests/resource/invalid/diagnostic`.
fn fixture(name: &str) -> String {
    let path = resource_path(false, &["diagnostic"], &format!("{name}.mamba"));
    fs::read_to_string(path).expect("read fixture")
}

/// The diagnostics of compiling the fixture `name` to Python.
fn diagnostics(name: &str) -> Vec<Diagnostic> {
    let source = Source::named(format!("{name}.mamba"), fixture(name));
    compile(&[source], &Options::default()).expect_err("the fixture does not compile")
}

#[test]
fn compile_str_to_python() {
    let python = compile_str(
        "def f(x: Int) -> Int := x + 1\nprint(f(2))\n",
        &Options::default(),
    )
    .expect("compiles");
    assert!(python.contains("def f(x"), "{python}");
    assert!(python.contains("print(f(2))"), "{python}");
}

#[test]
fn compile_str_to_asm() {
    let options = Options::new(Output::Asm {
        target: Some(String::from("x86_64-unknown-linux-gnu")),
    });
    let asm = compile_str("print(1 + 2)\n", &options).expect("compiles");
    assert!(asm.contains("; -- main --") && asm.contains("ret"), "{asm}");
}

#[test]
fn compile_to_object_gives_bytes() {
    let options = Options::new(Output::Object {
        target: Some(String::from("x86_64-unknown-linux-gnu")),
    });
    let artifacts = compile(&[Source::new("print(1)\n")], &options).expect("compiles");
    match artifacts.as_slice() {
        [Artifact::Bytes(bytes)] => assert_eq!(&bytes[..4], b"\x7fELF"),
        other => panic!("expected one object file, got {other:?}"),
    }
}

#[test]
fn compile_str_rejects_object_output() {
    let options = Options::new(Output::Object { target: None });
    let errs = compile_str("print(1)\n", &options).unwrap_err();
    assert_eq!(errs[0].stage, Stage::Generate);
}

#[test]
fn compile_gives_one_artifact_per_source() {
    let sources = [Source::new("print(1)\n"), Source::new("print(2)\n")];
    let artifacts = compile(&sources, &Options::default()).expect("compiles");
    assert_eq!(artifacts.len(), 2);
}

#[test]
fn version_is_the_crate_version() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}

#[test_case("lex", Stage::Lex, (2, 12), (2, 13))]
#[test_case("parse", Stage::Parse, (3, 17), (3, 17))]
#[test_case("type", Stage::Type, (4, 15), (4, 22))]
fn invalid_fixture_reports_stage_and_span(
    name: &str,
    stage: Stage,
    start: (usize, usize),
    end: (usize, usize),
) {
    let errs = diagnostics(name);
    assert_eq!(errs.len(), 1, "{errs:#?}");
    let error = &errs[0];

    assert_eq!(error.stage, stage);
    assert_eq!(error.source, Some(PathBuf::from(format!("{name}.mamba"))));
    let primary = error.primary_label().expect("a primary label");
    assert_eq!(primary.span.start, LineCol::new(start.0, start.1));
    assert_eq!(primary.span.end, LineCol::new(end.0, end.1));
}

#[test]
fn span_offsets_count_bytes_and_utf16() {
    // The emoji is four bytes, and two UTF-16 code units.
    let text = "let s := \"😀\"\nlet b: Int := \"x\"\n";
    let errs = compile_str(text, &Options::default()).unwrap_err();
    let span = errs[0].primary_label().expect("a primary label").span;

    let start = text.find("\"x\"").unwrap();
    assert_eq!(span.byte_start, start);
    assert_eq!(span.utf16_start, start - 2);
    assert_eq!(&text[span.byte_start..span.byte_end], "\"x\"");
}

#[test]
fn span_columns_count_characters_after_non_ascii_on_the_same_line() {
    // The core counts columns in bytes; the span counts characters.
    let text = "let b: Int := \"é\" + 1\n";
    let errs = compile_str(text, &Options::default()).unwrap_err();
    let span = errs[0].primary_label().expect("a primary label").span;

    assert_eq!(span.start, LineCol::new(1, 15));
    assert_eq!(&text[span.byte_start..span.byte_end], "\"é\"");
    assert_eq!(span.utf16_end - span.utf16_start, 3);
}

#[test]
fn a_parse_error_at_the_end_points_at_the_end() {
    let text = "def f(x: Int) -> Int := x +\n\n";
    let errs = compile_str(text, &Options::default()).unwrap_err();
    let primary = errs[0].primary_label().expect("a primary label");
    assert_eq!(primary.span.start, LineCol::new(1, 28));
}

#[test]
fn tokens_classify_and_include_comments() {
    let text = "def f(x: Int) -> Int := x + 1 # add one\nprint(\"a # b\")\n";
    let tokens = tokens(text).expect("lexes");
    let kinds: Vec<(TokenKind, &str)> = tokens
        .iter()
        .map(|token| {
            (
                token.kind,
                &text[token.span.byte_start..token.span.byte_end],
            )
        })
        .collect();

    assert_eq!(kinds[0], (TokenKind::Keyword, "def"));
    assert_eq!(kinds[1], (TokenKind::Identifier, "f"));
    assert!(kinds.contains(&(TokenKind::Operator, ":=")));
    assert!(kinds.contains(&(TokenKind::Number, "1")));
    assert!(kinds.contains(&(TokenKind::Comment, "# add one")));
    assert!(kinds.contains(&(TokenKind::String, "\"a # b\"")));
    assert_eq!(
        kinds
            .iter()
            .filter(|(kind, _)| *kind == TokenKind::Comment)
            .count(),
        1
    );
}

#[test]
fn tokens_of_a_string_over_lines_end_at_its_quote() {
    let text = "let s := \"one\ntwo\"\nprint(s)\n";
    let tokens = tokens(text).expect("lexes");
    let string = tokens
        .iter()
        .find(|token| token.kind == TokenKind::String)
        .unwrap();
    assert_eq!(
        &text[string.span.byte_start..string.span.byte_end],
        "\"one\ntwo\""
    );
}

#[test]
fn tokens_after_non_ascii_on_the_same_line_keep_their_place() {
    let text = "let s := \"é\" + \"a\" # ü\n";
    let tokens = tokens(text).expect("lexes");
    let texts: Vec<&str> = tokens
        .iter()
        .map(|token| &text[token.span.byte_start..token.span.byte_end])
        .collect();
    assert_eq!(texts, ["let", "s", ":=", "\"é\"", "+", "\"a\"", "# ü"]);
    assert_eq!(tokens[5].span.start, LineCol::new(1, 16));
}

#[test]
fn a_quote_in_an_interpolation_does_not_end_the_string() {
    let text = "print(\"a {f(\"x\")} b\") # done\n";
    let tokens = tokens(text).expect("lexes");
    let string = tokens
        .iter()
        .find(|token| token.kind == TokenKind::String)
        .unwrap();
    assert_eq!(
        &text[string.span.byte_start..string.span.byte_end],
        "\"a {f(\"x\")} b\""
    );
    assert_eq!(
        tokens
            .iter()
            .filter(|token| token.kind == TokenKind::Comment)
            .count(),
        1
    );
}

#[test]
fn tokens_report_a_lex_error() {
    let errs = tokens("let a := 1 $ 2").unwrap_err();
    assert_eq!(errs[0].stage, Stage::Lex);
}

#[cfg(feature = "serde")]
#[test]
fn diagnostic_round_trips_through_json() {
    let error = diagnostics("type").remove(0);
    let json = serde_json::to_string(&error).expect("serializes");
    assert!(json.contains("\"stage\":\"type\""), "{json}");
    assert!(json.contains("\"byteStart\""), "{json}");

    let back: Diagnostic = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(back, error);
}
