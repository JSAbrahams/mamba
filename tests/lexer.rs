use mamba::lexer::token::Token;
use mamba::lexer::token::TokenPos;
use mamba::lexer::tokenize;

#[test]
fn lex_from() {
    let source = String::from("from i use b");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { line: 1, pos: 1, token: Token::From },
        TokenPos { line: 1, pos: 6, token: Token::Id(String::from("i")) },
        TokenPos { line: 1, pos: 8, token: Token::Use },
        TokenPos { line: 1, pos: 12, token: Token::Id(String::from("b")) }
    ]);
}

#[test]
fn lex_operators() {
    let source = String::from("+ - * / ^ mod sqrt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { line: 1, pos: 1, token: Token::Add },
        TokenPos { line: 1, pos: 3, token: Token::Sub },
        TokenPos { line: 1, pos: 5, token: Token::Mul },
        TokenPos { line: 1, pos: 7, token: Token::Div },
        TokenPos { line: 1, pos: 9, token: Token::Pow },
        TokenPos { line: 1, pos: 11, token: Token::Mod },
        TokenPos { line: 1, pos: 15, token: Token::Sqrt },
        TokenPos { line: 1, pos: 20, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn lex_comparison() {
    let source = String::from("< > <= >= = /= is isnt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { line: 1, pos: 1, token: Token::Le },
        TokenPos { line: 1, pos: 3, token: Token::Ge },
        TokenPos { line: 1, pos: 5, token: Token::Leq },
        TokenPos { line: 1, pos: 8, token: Token::Geq },
        TokenPos { line: 1, pos: 11, token: Token::Eq },
        TokenPos { line: 1, pos: 13, token: Token::Neq },
        TokenPos { line: 1, pos: 16, token: Token::Is },
        TokenPos { line: 1, pos: 19, token: Token::IsN },
        TokenPos { line: 1, pos: 24, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn lex_if() {
    let source = String::from("if a then\n    b\nelse\n    c");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { line: 1, pos: 1, token: Token::If },
        TokenPos { line: 1, pos: 4, token: Token::Id(String::from("a")) },
        TokenPos { line: 1, pos: 6, token: Token::Then },
        TokenPos { line: 1, pos: 10, token: Token::NL },
        TokenPos { line: 2, pos: 1, token: Token::Indent },
        TokenPos { line: 2, pos: 5, token: Token::Id(String::from("b")) },
        TokenPos { line: 2, pos: 6, token: Token::NL },
        TokenPos { line: 3, pos: 1, token: Token::Dedent },
        TokenPos { line: 3, pos: 1, token: Token::Else },
        TokenPos { line: 3, pos: 5, token: Token::NL },
        TokenPos { line: 4, pos: 1, token: Token::Indent },
        TokenPos { line: 4, pos: 5, token: Token::Id(String::from("c")) }
    ]);
}

#[test]
fn lex_numbers_and_strings() {
    let source = String::from("1 2.0 3e10 5e 2 \"hello\" True False i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { line: 1, pos: 1, token: Token::Int(String::from("1")) },
        TokenPos { line: 1, pos: 3, token: Token::Real(String::from("2.0")) },
        TokenPos { line: 1, pos: 7, token: Token::ENum(String::from("3"), String::from("10")) },
        TokenPos { line: 1, pos: 12, token: Token::ENum(String::from("5"), String::new()) },
        TokenPos { line: 1, pos: 15, token: Token::Int(String::from("2")) },
        TokenPos { line: 1, pos: 17, token: Token::Str(String::from("hello")) },
        TokenPos { line: 1, pos: 25, token: Token::Bool(true) },
        TokenPos { line: 1, pos: 30, token: Token::Bool(false) },
        TokenPos { line: 1, pos: 36, token: Token::Id(String::from("i")) }
    ]);
}

#[test]
fn lex_identifiers() {
    let source = String::from("i _i 3a");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { line: 1, pos: 1, token: Token::Id(String::from("i")) },
        TokenPos { line: 1, pos: 3, token: Token::Id(String::from("_i")) },
        TokenPos { line: 1, pos: 6, token: Token::Int(String::from("3")) },
        TokenPos { line: 1, pos: 7, token: Token::Id(String::from("a")) },
    ]);
}

#[test]
fn lex_question() {
    let source = String::from("? ?. ?or i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        TokenPos { line: 1, pos: 1, token: Token::Quest },
        TokenPos { line: 1, pos: 3, token: Token::QuestCall },
        TokenPos { line: 1, pos: 6, token: Token::QuestOr },
        TokenPos { line: 1, pos: 10, token: Token::Id(String::from("i")) }
    ]);
}
