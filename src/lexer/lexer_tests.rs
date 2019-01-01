use super::*;

#[test]
fn assign_operator() {
    let source = "<-";
    let token = tokenize(source).unwrap();

    assert_eq!(vec![Token::Assign], token)
}

#[test]
fn simple_string() {
    let source = "\"test string\"";
    let token = tokenize(source).unwrap();

    assert_eq!(vec![Token::Str("test string".to_string())], token)
}

#[test]
fn natural_number() {
    let source = "123";
    let token = tokenize(source).unwrap();

    assert_eq!(vec![Token::Num(123.0)], token)
}

#[test]
fn float_number() {
    let source = "14.39";
    let token = tokenize(source).unwrap();

    assert_eq!(vec![Token::Num(14.39)], token)
}

#[test]
fn number_too_many_commas_gives_err() {
    let source = "14.39.12";
    assert!(tokenize(source).is_err());
}

#[test]
fn assign_number() {
    let source = "x <- 10";

    let tokens = tokenize(source).unwrap();
    assert_eq!(vec![Token::Id("x".to_string()), Token::Assign, Token::Num(10.0)], tokens)
}

#[test]
fn assign_no_spaces() {
    let source = "x<-10";

    let tokens = tokenize(source).unwrap();
    assert_eq!(vec![Token::Id("x".to_string()), Token::Assign, Token::Num(10.0)], tokens)
}

#[test]
fn assign_with_operators() {
    let source = "a <- (10 * b) +(y - c ) - (3 mod 20* 100) /\"hey\"";

    let tokens = tokenize(source).unwrap();
    assert_eq!(vec![
        Token::Id("a".to_string()), Token::Assign,
        Token::LPar, Token::Num(10.0), Token::Mul, Token::Id("b".to_string()), Token::RPar,
        Token::Add,
        Token::LPar, Token::Id("y".to_string()), Token::Sub, Token::Id("c".to_string()),
        Token::RPar, Token::Sub,
        Token::LPar, Token::Num(3.0), Token::Mod, Token::Num(20.0), Token::Mul, Token::Num(100.0),
        Token::RPar, Token::Div, Token::Str("hey".to_string())], tokens)
}
