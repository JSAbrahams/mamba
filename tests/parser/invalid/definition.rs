use mamba::lexer::tokenize;
use mamba::parser::parse_direct;

#[test]
fn def_multiple() {
    let source = String::from("def a b");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn def_mut_private_wrong_order() {
    let source = String::from("def mut private a ");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn def_missing_id() {
    let source = String::from("def");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn def_fun_no_closing_brack() {
    let source = String::from("def f(a");
    let err = parse_direct(&tokenize(&source).unwrap());
    assert_eq!(err.is_err(), true);
}

#[test]
fn def_fun_missing_arrow() {
    let source = String::from("def f(a) a * 10");
    let err = parse_direct(&tokenize(&source).unwrap());
    println!("{:?}", err);
    assert_eq!(err.is_err(), true);
}

#[test]
fn def_fun_missing_brackets() {
    let source = String::from("def f => print a");
    let err = parse_direct(&tokenize(&source).unwrap());
    println!("{:?}", err);
    assert_eq!(err.is_err(), true);
}
