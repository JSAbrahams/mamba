use mamba::common::position::{EndPoint, Position};
use mamba::lexer::token::Lex;
use mamba::lexer::token::Token;
use mamba::lexer::tokenize;

#[test]
fn from() {
    let source = String::from("from i import b");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        Lex {
            pos:   Position::new(&EndPoint::new(1, 1), &EndPoint::new(1, 5)),
            token: Token::From
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 6), &EndPoint::new(1, 7)),
            token: Token::Id(String::from("i"))
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 8), &EndPoint::new(1, 14)),
            token: Token::Import
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 15), &EndPoint::new(1, 16)),
            token: Token::Id(String::from("b"))
        }
    ]);
}

#[test]
fn operators() {
    let source = String::from("+ - * / ^ mod sqrt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        Lex { pos: Position::new(&EndPoint::new(1, 1), &EndPoint::new(1, 2)), token: Token::Add },
        Lex { pos: Position::new(&EndPoint::new(1, 3), &EndPoint::new(1, 4)), token: Token::Sub },
        Lex { pos: Position::new(&EndPoint::new(1, 5), &EndPoint::new(1, 6)), token: Token::Mul },
        Lex { pos: Position::new(&EndPoint::new(1, 7), &EndPoint::new(1, 8)), token: Token::Div },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 9), &EndPoint::new(1, 10)),
            token: Token::Pow
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 11), &EndPoint::new(1, 14)),
            token: Token::Mod
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 15), &EndPoint::new(1, 19)),
            token: Token::Sqrt
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 20), &EndPoint::new(1, 21)),
            token: Token::Id(String::from("i"))
        }
    ]);
}

#[test]
fn comparison() {
    let source = String::from("< > <= >= = /= is isnt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        Lex { pos: Position::new(&EndPoint::new(1, 1), &EndPoint::new(1, 2)), token: Token::Le },
        Lex { pos: Position::new(&EndPoint::new(1, 3), &EndPoint::new(1, 4)), token: Token::Ge },
        Lex { pos: Position::new(&EndPoint::new(1, 5), &EndPoint::new(1, 7)), token: Token::Leq },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 8), &EndPoint::new(1, 10)),
            token: Token::Geq
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 11), &EndPoint::new(1, 12)),
            token: Token::Eq
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 13), &EndPoint::new(1, 15)),
            token: Token::Neq
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 16), &EndPoint::new(1, 18)),
            token: Token::Is
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 19), &EndPoint::new(1, 23)),
            token: Token::IsN
        },
        Lex {
            pos:   Position::new(&EndPoint::new(1, 24), &EndPoint::new(1, 25)),
            token: Token::Id(String::from("i"))
        }
    ]);
}
