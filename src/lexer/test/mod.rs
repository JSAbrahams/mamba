use super::*;

#[test]
fn assign_operator() {
    let source = "<-";
    let token = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::Assign], token)
}

#[test]
fn simple_string() {
    let source = "\"test string\"";
    let token = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::Str("test string".to_string())], token)
}

#[test]
fn natural_number() {
    let source = "123";
    let token = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::Int(123.to_string())], token)
}

#[test]
fn float_number() {
    let source = "14.39";
    let tokens = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::Real(14.39.to_string())], tokens)
}

#[test]
fn e_number() {
    let source = "14e30";
    let tokens = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::ENum(14.to_string(), 30.to_string())], tokens)
}

#[test]
fn e_number_float() {
    let source = "14.39e30";
    let tokens = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::ENum(14.39.to_string(), 30.to_string())], tokens)
}

#[test]
fn e_number_without_num() {
    let source = ".e10";
    let tokens = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::ENum(0.to_string(), 10.to_string())], tokens)
}

#[test]
fn float_and_e_number() {
    let source = "14.39.e30";
    let tokens = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::Real(14.39.to_string()), Token::ENum(0.to_string(),
                                                                30.to_string())], tokens)
}

#[test]
fn float_and_e_number_and_float() {
    let source = "14.39.e30.0";
    let tokens = tokenize(source.to_string()).unwrap();

    assert_eq!(vec![Token::Real(14.39.to_string()),
                    Token::ENum(0.to_string(), 30.to_string()),
                    Token::Real(0.to_string())], tokens)
}

#[test]
fn assign_number() {
    let source = "x <- 10";

    let tokens = tokenize(source.to_string()).unwrap();
    assert_eq!(vec![Token::Id("x".to_string()), Token::Assign, Token::Int(10.to_string())], tokens)
}

#[test]
fn assign_no_spaces() {
    let source = "x<-10";

    let tokens = tokenize(source.to_string()).unwrap();
    assert_eq!(vec![Token::Id("x".to_string()), Token::Assign, Token::Int(10.to_string())], tokens)
}

#[test]
fn assign_with_operators() {
    let source = "a <- (10.5 * b) +(y - c ) - (3 mod 20* 100) /\"hey\"";

    let tokens = tokenize(source.to_string()).unwrap();
    assert_eq!(vec![
        Token::Id("a".to_string()), Token::Assign,
        Token::LPar, Token::Real(10.5.to_string()), Token::Mul, Token::Id("b".to_string()), Token::RPar,
        Token::Add,
        Token::LPar, Token::Id("y".to_string()), Token::Sub, Token::Id("c".to_string()),
        Token::RPar, Token::Sub,
        Token::LPar, Token::Int(3.to_string()), Token::Mod, Token::Int(20.to_string()), Token::Mul,
        Token::Int(100.to_string()), Token::RPar, Token::Div, Token::Str("hey".to_string())],
               tokens)
}
