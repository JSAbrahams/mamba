use super::*;

#[test]
fn get_assign_operator() {
    let source = "<-";
    let token = get_operator(&mut source.chars().peekable()).unwrap();

    assert_eq!(Token::ASSIGN, token)
}

#[test]
fn get_simple_string() {
    let source = "\"test string\"";
    let token = get_string(&mut source.chars().peekable()).unwrap();

    assert_eq!(Token::String("test string".to_string()), token)
}

#[test]
fn get_natural_number() {
    let source = "123";
    let token = get_number(&mut source.chars().peekable()).unwrap();

    assert_eq!(Token::Num(123.0), token)
}

#[test]
fn get_number_float() {
    let source = "14.39";
    let token = get_number(&mut source.chars().peekable()).unwrap();

    assert_eq!(Token::Num(14.39), token)
}

#[test]
fn get_number_too_many_commas_gives_err() {
    let source = "14.39.12";
    assert!(get_number(&mut source.chars().peekable()).is_err());
}

#[test]
fn simple_assign() {
    let source = "x <- 10";

    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens, vec![Token::Id("x".to_string()), Token::ASSIGN, Token::Num(10.0)])
}

#[test]
fn simple_assign_no_spaces() {
    let source = "x<-10";

    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens, vec![Token::Id("x".to_string()), Token::ASSIGN, Token::Num(10.0)])
}

#[test]
fn assign_with_operators() {
    let source = "a <- (10 * b) +(y - c ) - (3 mod 20* 100)";

    let tokens = tokenize(source).unwrap();
    assert_eq!(tokens, vec![
        Token::Id("x".to_string()), Token::ASSIGN,
        Token::LPAREN, Token::Num(10.0), Token::MUL, Token::Id("b".to_string()), Token::RPAREN,
        Token::ADD,
        Token::LPAREN, Token::Id("y".to_string()), Token::SUB, Token::Id("c".to_string()),
        Token::RPAREN, Token::SUB,
        Token::LPAREN, Token::Num(3.0), Token::MOD, Token::Num(20.0), Token::MUL, Token::RPAREN
    ])
}
