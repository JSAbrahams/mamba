use mamba::lexer::tokenize;
use mamba::parser::parse;

#[test]
fn direct_call_missing_closing_bracket() {
    let source = String::from("a(b");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn regular_call_missing_closing_bracket() {
    let source = String::from("instance.a(b");
    parse(&tokenize(&source).unwrap()).unwrap_err();
}
