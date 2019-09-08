use mamba::common::position::EndPoint;
use mamba::lexer::token::Token;
use mamba::lexer::token::TokenPos;
use mamba::lexer::tokenize;

#[test]
fn from() {
    let source = String::from("from i import b");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::From },
        TokenPos { start: EndPoint { line: 1, pos: 6 }, token: Token::Id(String::from("i")) },
        TokenPos { start: EndPoint { line: 1, pos: 8 }, token: Token::Import },
        TokenPos { start: EndPoint { line: 1, pos: 15 }, token: Token::Id(String::from("b")) }
    ]);
}

#[test]
fn operators() {
    let source = String::from("+ - * / ^ mod sqrt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::Add },
        TokenPos { start: EndPoint { line: 1, pos: 3 }, token: Token::Sub },
        TokenPos { start: EndPoint { line: 1, pos: 5 }, token: Token::Mul },
        TokenPos { start: EndPoint { line: 1, pos: 7 }, token: Token::Div },
        TokenPos { start: EndPoint { line: 1, pos: 9 }, token: Token::Pow },
        TokenPos { start: EndPoint { line: 1, pos: 11 }, token: Token::Mod },
        TokenPos { start: EndPoint { line: 1, pos: 15 }, token: Token::Sqrt },
        TokenPos { start: EndPoint { line: 1, pos: 20 }, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn comparison() {
    let source = String::from("< > <= >= = /= is isnt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::Le },
        TokenPos { start: EndPoint { line: 1, pos: 3 }, token: Token::Ge },
        TokenPos { start: EndPoint { line: 1, pos: 5 }, token: Token::Leq },
        TokenPos { start: EndPoint { line: 1, pos: 8 }, token: Token::Geq },
        TokenPos { start: EndPoint { line: 1, pos: 11 }, token: Token::Eq },
        TokenPos { start: EndPoint { line: 1, pos: 13 }, token: Token::Neq },
        TokenPos { start: EndPoint { line: 1, pos: 16 }, token: Token::Is },
        TokenPos { start: EndPoint { line: 1, pos: 19 }, token: Token::IsN },
        TokenPos { start: EndPoint { line: 1, pos: 24 }, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn lex_if() {
    let source = String::from("if a then\n    b\nelse\n    c");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::If },
        TokenPos { start: EndPoint { line: 1, pos: 4 }, token: Token::Id(String::from("a")) },
        TokenPos { start: EndPoint { line: 1, pos: 6 }, token: Token::Then },
        TokenPos { start: EndPoint { line: 1, pos: 10 }, token: Token::NL },
        TokenPos { start: EndPoint { line: 2, pos: 5 }, token: Token::Indent },
        TokenPos { start: EndPoint { line: 2, pos: 5 }, token: Token::Id(String::from("b")) },
        TokenPos { start: EndPoint { line: 2, pos: 6 }, token: Token::NL },
        TokenPos { start: EndPoint { line: 3, pos: 1 }, token: Token::Dedent },
        TokenPos { start: EndPoint { line: 3, pos: 1 }, token: Token::Else },
        TokenPos { start: EndPoint { line: 3, pos: 5 }, token: Token::NL },
        TokenPos { start: EndPoint { line: 4, pos: 5 }, token: Token::Indent },
        TokenPos { start: EndPoint { line: 4, pos: 5 }, token: Token::Id(String::from("c")) },
        TokenPos { start: EndPoint { line: 4, pos: 6 }, token: Token::Dedent }
    ]);
}

#[test]
fn numbers_and_strings() {
    let source = String::from("1 2.0 3E10 5E 2 \"hello\" True False i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::Int(String::from("1")) },
        TokenPos { start: EndPoint { line: 1, pos: 3 }, token: Token::Real(String::from("2.0")) },
        TokenPos {
            start: EndPoint { line: 1, pos: 7 },
            token: Token::ENum(String::from("3"), String::from("10"))
        },
        TokenPos {
            start: EndPoint { line: 1, pos: 12 },
            token: Token::ENum(String::from("5"), String::new())
        },
        TokenPos { start: EndPoint { line: 1, pos: 15 }, token: Token::Int(String::from("2")) },
        TokenPos { start: EndPoint { line: 1, pos: 17 }, token: Token::Str(String::from("hello")) },
        TokenPos { start: EndPoint { line: 1, pos: 25 }, token: Token::Bool(true) },
        TokenPos { start: EndPoint { line: 1, pos: 30 }, token: Token::Bool(false) },
        TokenPos { start: EndPoint { line: 1, pos: 36 }, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn identifiers() {
    let source = String::from("i _i 3a");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::Id(String::from("i")) },
        TokenPos { start: EndPoint { line: 1, pos: 3 }, token: Token::Id(String::from("_i")) },
        TokenPos { start: EndPoint { line: 1, pos: 6 }, token: Token::Int(String::from("3")) },
        TokenPos { start: EndPoint { line: 1, pos: 7 }, token: Token::Id(String::from("a")) },
    ]);
}

#[test]
fn question() {
    let source = String::from("? ?. i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::Question },
        TokenPos { start: EndPoint { line: 1, pos: 3 }, token: Token::QuestCall },
        TokenPos { start: EndPoint { line: 1, pos: 6 }, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn pass_undefined_underscore() {
    let source = String::from("pass undefined _");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::Pass },
        TokenPos { start: EndPoint { line: 1, pos: 6 }, token: Token::Undefined },
        TokenPos { start: EndPoint { line: 1, pos: 16 }, token: Token::Underscore }
    ]);
}

#[test]
fn bitwise_operators() {
    let source = String::from("_and_ _or_ _xor_ _not_ << >>");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { start: EndPoint { line: 1, pos: 1 }, token: Token::BAnd },
        TokenPos { start: EndPoint { line: 1, pos: 7 }, token: Token::BOr },
        TokenPos { start: EndPoint { line: 1, pos: 12 }, token: Token::BXOr },
        TokenPos { start: EndPoint { line: 1, pos: 18 }, token: Token::BOneCmpl },
        TokenPos { start: EndPoint { line: 1, pos: 24 }, token: Token::BLShift },
        TokenPos { start: EndPoint { line: 1, pos: 27 }, token: Token::BRShift },
    ]);
}
