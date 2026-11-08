//! The public API of the crate as a library: compiling text, and diagnostics as data.
//!
//! Nothing here needs a feature, except the JSON round trip, which needs `serde`.

use std::fs;
use std::path::PathBuf;

use test_case::test_case;

use mamba::{
    compile, compile_str, version, Artifact, Diagnostic, LineCol, Options, Output, Source, Stage,
};
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

/// The generated Python never has two empty lines in a row, which classes used to leave behind.
#[test]
fn python_has_at_most_one_empty_line_in_a_row() {
    let source = "class A where\n    def f(self) -> Int := 1\nend\n\nclass B where\n    def g(self) -> Int := 2\nend\n\nlet a := A.new()\nprint(a.f())\n";
    let python = compile_str(source, &Options::default()).expect("compiles");

    let empty: Vec<bool> = python.lines().map(|line| line.trim().is_empty()).collect();
    let consecutive = empty.windows(2).any(|pair| pair[0] && pair[1]);
    assert!(!consecutive, "{python}");
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

/// The version comes from `git describe --tags`, or is the crate version without git.
///
/// A shallow clone may have no tags, so the exact string is not checked.
#[test]
fn version_is_a_tag_or_the_crate_version() {
    let version = version();
    assert!(!version.is_empty());
    assert!(
        version.starts_with(env!("CARGO_PKG_VERSION")) || version.starts_with('v'),
        "{version}"
    );
}

/// The parser reports an error of the lexer, such as an unrecognized character, as its own.
#[test_case("lex", Stage::Parse, (2, 12), (2, 13))]
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

/// The emoji is four bytes, and one character.
#[test]
fn span_offsets_count_bytes() {
    let text = "let s := \"😀\"\nlet b: Int := \"x\"\n";
    let errs = compile_str(text, &Options::default()).unwrap_err();
    let span = errs[0].primary_label().expect("a primary label").span;

    let start = text.find("\"x\"").unwrap();
    assert_eq!(span.byte_start, start);
    assert_eq!(&text[span.byte_start..span.byte_end], "\"x\"");
    assert_eq!(
        (span.start, span.end),
        (LineCol::new(2, 15), LineCol::new(2, 18))
    );
}

/// The core counts columns in bytes, and the span counts characters.
#[test]
fn span_columns_count_characters_after_non_ascii_on_the_same_line() {
    let text = "let b: Int := \"é\" + 1\n";
    let errs = compile_str(text, &Options::default()).unwrap_err();
    let span = errs[0].primary_label().expect("a primary label").span;

    assert_eq!(span.start, LineCol::new(1, 15));
    assert_eq!(span.end, LineCol::new(1, 18));
    assert_eq!(&text[span.byte_start..span.byte_end], "\"é\"");
}

#[test]
fn a_parse_error_at_the_end_points_at_the_end() {
    let text = "def f(x: Int) -> Int := x +\n\n";
    let errs = compile_str(text, &Options::default()).unwrap_err();
    let primary = errs[0].primary_label().expect("a primary label");
    assert_eq!(primary.span.start, LineCol::new(1, 28));
}

/// A construct the machine code backend does not support is named, not dumped.
#[test]
fn unsupported_construct_is_named_in_a_short_message() {
    let text = "def f(x: Int) -> Int := match x where\n    0 => 1\n    n => n\nend\nprint(f(1))\n";
    let options = Options::new(Output::Asm {
        target: Some(String::from("x86_64-unknown-linux-gnu")),
    });
    let errs = compile_str(text, &options).unwrap_err();
    let error = &errs[0];

    assert_eq!(error.stage, Stage::Generate);
    assert!(error.message.len() < 200, "{}", error.message);
    assert!(error.message.contains("match"), "{}", error.message);
    let primary = error.primary_label().expect("a primary label");
    assert_eq!(primary.span.start.line, 1);
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
