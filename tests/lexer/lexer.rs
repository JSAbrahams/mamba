use mamba::lexer::token::Token;
use mamba::lexer::token::TokenPos;
use mamba::lexer::tokenize;

#[test]
fn from() {
    let source = String::from("from i import b");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::From },
        TokenPos { st_line: 1, st_pos: 6, token: Token::Id(String::from("i")) },
        TokenPos { st_line: 1, st_pos: 8, token: Token::Import },
        TokenPos { st_line: 1, st_pos: 15, token: Token::Id(String::from("b")) }
    ]);
}

#[test]
fn operators() {
    let source = String::from("+ - * / ^ mod sqrt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::Add },
        TokenPos { st_line: 1, st_pos: 3, token: Token::Sub },
        TokenPos { st_line: 1, st_pos: 5, token: Token::Mul },
        TokenPos { st_line: 1, st_pos: 7, token: Token::Div },
        TokenPos { st_line: 1, st_pos: 9, token: Token::Pow },
        TokenPos { st_line: 1, st_pos: 11, token: Token::Mod },
        TokenPos { st_line: 1, st_pos: 15, token: Token::Sqrt },
        TokenPos { st_line: 1, st_pos: 20, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn comparison() {
    let source = String::from("< > <= >= = /= is isnt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::Le },
        TokenPos { st_line: 1, st_pos: 3, token: Token::Ge },
        TokenPos { st_line: 1, st_pos: 5, token: Token::Leq },
        TokenPos { st_line: 1, st_pos: 8, token: Token::Geq },
        TokenPos { st_line: 1, st_pos: 11, token: Token::Eq },
        TokenPos { st_line: 1, st_pos: 13, token: Token::Neq },
        TokenPos { st_line: 1, st_pos: 16, token: Token::Is },
        TokenPos { st_line: 1, st_pos: 19, token: Token::IsN },
        TokenPos { st_line: 1, st_pos: 24, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn lex_if() {
    let source = String::from("if a then\n    b\nelse\n    c");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::If },
        TokenPos { st_line: 1, st_pos: 4, token: Token::Id(String::from("a")) },
        TokenPos { st_line: 1, st_pos: 6, token: Token::Then },
        TokenPos { st_line: 1, st_pos: 10, token: Token::NL },
        TokenPos { st_line: 2, st_pos: 5, token: Token::Indent },
        TokenPos { st_line: 2, st_pos: 5, token: Token::Id(String::from("b")) },
        TokenPos { st_line: 2, st_pos: 6, token: Token::NL },
        TokenPos { st_line: 3, st_pos: 1, token: Token::Dedent },
        TokenPos { st_line: 3, st_pos: 1, token: Token::Else },
        TokenPos { st_line: 3, st_pos: 5, token: Token::NL },
        TokenPos { st_line: 4, st_pos: 5, token: Token::Indent },
        TokenPos { st_line: 4, st_pos: 5, token: Token::Id(String::from("c")) },
        TokenPos { st_line: 4, st_pos: 5, token: Token::Dedent }
    ]);
}

#[test]
fn numbers_and_strings() {
    let source = String::from("1 2.0 3E10 5E 2 \"hello\" True False i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::Int(String::from("1")) },
        TokenPos { st_line: 1, st_pos: 3, token: Token::Real(String::from("2.0")) },
        TokenPos {
            st_line: 1,
            st_pos:  7,
            token:   Token::ENum(String::from("3"), String::from("10"))
        },
        TokenPos {
            st_line: 1,
            st_pos:  12,
            token:   Token::ENum(String::from("5"), String::new())
        },
        TokenPos { st_line: 1, st_pos: 15, token: Token::Int(String::from("2")) },
        TokenPos { st_line: 1, st_pos: 17, token: Token::Str(String::from("hello")) },
        TokenPos { st_line: 1, st_pos: 25, token: Token::Bool(true) },
        TokenPos { st_line: 1, st_pos: 30, token: Token::Bool(false) },
        TokenPos { st_line: 1, st_pos: 36, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn identifiers() {
    let source = String::from("i _i 3a");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::Id(String::from("i")) },
        TokenPos { st_line: 1, st_pos: 3, token: Token::Id(String::from("_i")) },
        TokenPos { st_line: 1, st_pos: 6, token: Token::Int(String::from("3")) },
        TokenPos { st_line: 1, st_pos: 7, token: Token::Id(String::from("a")) },
    ]);
}

#[test]
fn question() {
    let source = String::from("? ?. ?or i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::Quest },
        TokenPos { st_line: 1, st_pos: 3, token: Token::QuestCall },
        TokenPos { st_line: 1, st_pos: 6, token: Token::QuestOr },
        TokenPos { st_line: 1, st_pos: 10, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn pass_undefined_underscore() {
    let source = String::from("pass undefined _");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::Pass },
        TokenPos { st_line: 1, st_pos: 6, token: Token::Undefined },
        TokenPos { st_line: 1, st_pos: 16, token: Token::Underscore }
    ]);
}

#[test]
fn bitwise_operators() {
    let source = String::from("_and_ _or_ _xor_ _not_ << >>");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { st_line: 1, st_pos: 1, token: Token::BAnd },
        TokenPos { st_line: 1, st_pos: 7, token: Token::BOr },
        TokenPos { st_line: 1, st_pos: 12, token: Token::BXOr },
        TokenPos { st_line: 1, st_pos: 18, token: Token::BOneCmpl },
        TokenPos { st_line: 1, st_pos: 24, token: Token::BLShift },
        TokenPos { st_line: 1, st_pos: 27, token: Token::BRShift },
    ]);
}
