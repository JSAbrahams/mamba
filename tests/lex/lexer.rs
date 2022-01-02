use mamba::common::position::{CaretPos, Position};
use mamba::lex::token::Lex;
use mamba::lex::token::Token;
use mamba::lex::tokenize;

#[test]
fn from() {
    let source = String::from("from i import b");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        Lex {
            pos: Position::new(&CaretPos::new(1, 1), &CaretPos::new(1, 5)),
            token: Token::From,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 6), &CaretPos::new(1, 7)),
            token: Token::Id(String::from("i")),
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 8), &CaretPos::new(1, 14)),
            token: Token::Import,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 15), &CaretPos::new(1, 16)),
            token: Token::Id(String::from("b")),
        },
    ]);
}

#[test]
fn operators() {
    let source = String::from("+ - * / ^ mod sqrt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        Lex { pos: Position::new(&CaretPos::new(1, 1), &CaretPos::new(1, 2)), token: Token::Add },
        Lex { pos: Position::new(&CaretPos::new(1, 3), &CaretPos::new(1, 4)), token: Token::Sub },
        Lex { pos: Position::new(&CaretPos::new(1, 5), &CaretPos::new(1, 6)), token: Token::Mul },
        Lex { pos: Position::new(&CaretPos::new(1, 7), &CaretPos::new(1, 8)), token: Token::Div },
        Lex {
            pos: Position::new(&CaretPos::new(1, 9), &CaretPos::new(1, 10)),
            token: Token::Pow,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 11), &CaretPos::new(1, 14)),
            token: Token::Mod,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 15), &CaretPos::new(1, 19)),
            token: Token::Sqrt,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 20), &CaretPos::new(1, 21)),
            token: Token::Id(String::from("i")),
        },
    ]);
}

#[test]
fn comparison() {
    let source = String::from("< > <= >= = /= is isnt i");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![
        Lex { pos: Position::new(&CaretPos::new(1, 1), &CaretPos::new(1, 2)), token: Token::Le },
        Lex { pos: Position::new(&CaretPos::new(1, 3), &CaretPos::new(1, 4)), token: Token::Ge },
        Lex { pos: Position::new(&CaretPos::new(1, 5), &CaretPos::new(1, 7)), token: Token::Leq },
        Lex {
            pos: Position::new(&CaretPos::new(1, 8), &CaretPos::new(1, 10)),
            token: Token::Geq,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 11), &CaretPos::new(1, 12)),
            token: Token::Eq,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 13), &CaretPos::new(1, 15)),
            token: Token::Neq,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 16), &CaretPos::new(1, 18)),
            token: Token::Is,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 19), &CaretPos::new(1, 23)),
            token: Token::IsN,
        },
        Lex {
            pos: Position::new(&CaretPos::new(1, 24), &CaretPos::new(1, 25)),
            token: Token::Id(String::from("i")),
        },
    ]);
}

#[test]
fn fstring() {
    let source = String::from("\"my string {my_var}\"");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![Lex {
        pos: Position::new(&CaretPos::new(1, 1), &CaretPos::new(1, 21)),
        token: Token::Str(String::from("my string {my_var}"), vec![vec![Lex {
            pos: Position::new(&CaretPos::new(1, 13), &CaretPos::new(1, 19)),
            token: Token::Id(String::from("my_var")),
        }]], ),
    }, ]);
}

#[test]
fn fstring_set() {
    let source = String::from("\"{{a, b}}\"");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![Lex {
        pos: Position::new(&CaretPos::new(1, 1), &CaretPos::new(1, 11)),
        token: Token::Str(String::from("{{a, b}}"), vec![vec![
            Lex {
                pos: Position::new(&CaretPos::new(1, 3), &CaretPos::new(1, 4)),
                token: Token::LCBrack,
            },
            Lex {
                pos: Position::new(&CaretPos::new(1, 4), &CaretPos::new(1, 5)),
                token: Token::Id(String::from("a")),
            },
            Lex {
                pos: Position::new(&CaretPos::new(1, 5), &CaretPos::new(1, 6)),
                token: Token::Comma,
            },
            Lex {
                pos: Position::new(&CaretPos::new(1, 7), &CaretPos::new(1, 8)),
                token: Token::Id(String::from("b")),
            },
            Lex {
                pos: Position::new(&CaretPos::new(1, 8), &CaretPos::new(1, 9)),
                token: Token::RCBrack,
            },
        ]]),
    }, ]);
}

#[test]
fn fstring_operation() {
    let source = String::from("\"{a + b}\"");
    let tokens = tokenize(&source).unwrap();
    assert_eq!(tokens, vec![Lex {
        pos: Position::new(&CaretPos::new(1, 1), &CaretPos::new(1, 10)),
        token: Token::Str(String::from("{a + b}"), vec![vec![
            Lex {
                pos: Position::new(&CaretPos::new(1, 3), &CaretPos::new(1, 4)),
                token: Token::Id(String::from("a")),
            },
            Lex {
                pos: Position::new(&CaretPos::new(1, 5), &CaretPos::new(1, 6)),
                token: Token::Add,
            },
            Lex {
                pos: Position::new(&CaretPos::new(1, 7), &CaretPos::new(1, 8)),
                token: Token::Id(String::from("b")),
            },
        ]]),
    }, ]);
}
