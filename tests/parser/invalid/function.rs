use mamba::lexer::tokenize;
use mamba::parser::parse_direct;

#[test]
fn direct_call_missing_closing_bracket() {
    let source = String::from("a(b");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn regular_call_missing_closing_bracket() {
    let source = String::from("instance.a(b");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}
