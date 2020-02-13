use mamba::lex::tokenize;
use mamba::parse::parse_direct;

#[test]
fn handle_no_branches() {
    let source = String::from("def a handle");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}

#[test]
fn handle_no_indentation() {
    let source = String::from("def a handle\nerr: Err => b");
    parse_direct(&tokenize(&source).unwrap()).unwrap_err();
}
