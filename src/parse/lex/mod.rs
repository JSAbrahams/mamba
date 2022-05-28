use crate::common::position::CaretPos;
use crate::parse::lex::pass::pass;
use crate::parse::lex::result::LexResult;
use crate::parse::lex::state::State;
use crate::parse::lex::token::{Lex, Token};
use crate::parse::lex::tokenize::into_tokens;

pub mod result;
pub mod token;

#[macro_use]
mod state;
mod pass;
mod tokenize;

/// Convert a given string to a sequence of
/// [TokenPos](mamba::lexer::token::TokenPos), each containing a
/// [Token](mamba::lexer::token::Token), in addition to line number and
/// position. Note that line number and position are 1-indexed.
///
/// Should never panic.
#[allow(clippy::while_let_on_iterator)]
pub fn tokenize(input: &str) -> LexResult {
    let mut tokens = Vec::new();
    let mut state = State::new();

    let mut it = input.chars().peekable();
    while let Some(c) = it.next() {
        tokens.append(&mut into_tokens(c, &mut it, &mut state)?);
    }
    tokens.append(&mut state.flush_indents());
    tokens.push(Lex::new(
        if let Some(lex) = tokens.last() { lex.pos.end.offset_pos(1) } else { CaretPos::default() },
        Token::Eof,
    ));

    let tokens = pass(&tokens);
    Ok(tokens)
}

fn tokenize_direct(input: &str) -> LexResult {
    let mut tokens = Vec::new();
    let mut state = State::new();

    let mut it = input.chars().peekable();
    while let Some(c) = it.next() {
        tokens.append(&mut into_tokens(c, &mut it, &mut state)?);
    }
    tokens.append(&mut state.flush_indents());

    let tokens = pass(&tokens);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::common::position::{CaretPos, Position};
    use crate::parse::lex::token::{Lex, Token};
    use crate::parse::lex::tokenize;

    #[test]
    fn from() {
        let source = String::from("from i import b");
        let tokens = tokenize(&source).unwrap();
        assert_eq!(
            tokens,
            vec![
                Lex {
                    pos: Position::new(CaretPos::new(1, 1), CaretPos::new(1, 5)),
                    token: Token::From,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 6), CaretPos::new(1, 7)),
                    token: Token::Id(String::from("i")),
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 8), CaretPos::new(1, 14)),
                    token: Token::Import,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 15), CaretPos::new(1, 16)),
                    token: Token::Id(String::from("b")),
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 17), CaretPos::new(1, 20)),
                    token: Token::Eof,
                },
            ]
        );
    }

    #[test]
    fn exclamation_invalid() {
        let source = String::from("!");
        assert!(tokenize(&source).is_err());
    }

    #[test]
    fn assign_operations() {
        let source = String::from(":= += -= *= /= ^= >>= <<=");
        let tokens = tokenize(&source).unwrap();
        assert_eq!(
            tokens.iter().map(|l| l.token.clone()).collect_vec(),
            vec![
                Token::Assign,
                Token::AddAssign,
                Token::SubAssign,
                Token::MulAssign,
                Token::DivAssign,
                Token::PowAssign,
                Token::BRShiftAssign,
                Token::BLShiftAssign,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn operators() {
        let source = String::from("+ - * / ^ mod sqrt i");
        let tokens = tokenize(&source).unwrap();
        assert_eq!(
            tokens,
            vec![
                Lex {
                    pos: Position::new(CaretPos::new(1, 1), CaretPos::new(1, 2)),
                    token: Token::Add,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 3), CaretPos::new(1, 4)),
                    token: Token::Sub,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 5), CaretPos::new(1, 6)),
                    token: Token::Mul,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 7), CaretPos::new(1, 8)),
                    token: Token::Div,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 9), CaretPos::new(1, 10)),
                    token: Token::Pow,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 11), CaretPos::new(1, 14)),
                    token: Token::Mod,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 15), CaretPos::new(1, 19)),
                    token: Token::Sqrt,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 20), CaretPos::new(1, 21)),
                    token: Token::Id(String::from("i")),
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 22), CaretPos::new(1, 25)),
                    token: Token::Eof,
                },
            ]
        );
    }

    #[test]
    fn comparison() {
        let source = String::from("< > <= >= = != is isnt i");
        let tokens = tokenize(&source).unwrap();
        assert_eq!(
            tokens,
            vec![
                Lex {
                    pos: Position::new(CaretPos::new(1, 1), CaretPos::new(1, 2)),
                    token: Token::Le,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 3), CaretPos::new(1, 4)),
                    token: Token::Ge,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 5), CaretPos::new(1, 7)),
                    token: Token::Leq,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 8), CaretPos::new(1, 10)),
                    token: Token::Geq,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 11), CaretPos::new(1, 12)),
                    token: Token::Eq,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 13), CaretPos::new(1, 15)),
                    token: Token::Neq,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 16), CaretPos::new(1, 18)),
                    token: Token::Is,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 19), CaretPos::new(1, 23)),
                    token: Token::IsN,
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 24), CaretPos::new(1, 25)),
                    token: Token::Id(String::from("i")),
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 26), CaretPos::new(1, 29)),
                    token: Token::Eof,
                },
            ]
        );
    }

    #[test]
    fn fstring() {
        let source = String::from("\"my string {my_var}\"");
        let tokens = tokenize(&source).unwrap();
        assert_eq!(
            tokens,
            vec![
                Lex {
                    pos: Position::new(CaretPos::new(1, 1), CaretPos::new(1, 21)),
                    token: Token::Str(
                        String::from("my string {my_var}"),
                        vec![vec![Lex {
                            pos: Position::new(CaretPos::new(1, 13), CaretPos::new(1, 19)),
                            token: Token::Id(String::from("my_var")),
                        }]],
                    ),
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 22), CaretPos::new(1, 25)),
                    token: Token::Eof,
                },
            ]
        );
    }

    #[test]
    fn fstring_set() {
        let source = String::from("\"{{a, b}}\"");
        let tokens = tokenize(&source).unwrap();
        assert_eq!(
            tokens,
            vec![
                Lex {
                    pos: Position::new(CaretPos::new(1, 1), CaretPos::new(1, 11)),
                    token: Token::Str(
                        String::from("{{a, b}}"),
                        vec![vec![
                            Lex {
                                pos: Position::new(CaretPos::new(1, 3), CaretPos::new(1, 4)),
                                token: Token::LCBrack,
                            },
                            Lex {
                                pos: Position::new(CaretPos::new(1, 4), CaretPos::new(1, 5)),
                                token: Token::Id(String::from("a")),
                            },
                            Lex {
                                pos: Position::new(CaretPos::new(1, 5), CaretPos::new(1, 6)),
                                token: Token::Comma,
                            },
                            Lex {
                                pos: Position::new(CaretPos::new(1, 7), CaretPos::new(1, 8)),
                                token: Token::Id(String::from("b")),
                            },
                            Lex {
                                pos: Position::new(CaretPos::new(1, 8), CaretPos::new(1, 9)),
                                token: Token::RCBrack,
                            },
                        ]],
                    ),
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 12), CaretPos::new(1, 15)),
                    token: Token::Eof,
                },
            ]
        );
    }

    #[test]
    fn fstring_operation() {
        let source = String::from("\"{a + b}\"");
        let tokens = tokenize(&source).unwrap();
        assert_eq!(
            tokens,
            vec![
                Lex {
                    pos: Position::new(CaretPos::new(1, 1), CaretPos::new(1, 10)),
                    token: Token::Str(
                        String::from("{a + b}"),
                        vec![vec![
                            Lex {
                                pos: Position::new(CaretPos::new(1, 3), CaretPos::new(1, 4)),
                                token: Token::Id(String::from("a")),
                            },
                            Lex {
                                pos: Position::new(CaretPos::new(1, 5), CaretPos::new(1, 6)),
                                token: Token::Add,
                            },
                            Lex {
                                pos: Position::new(CaretPos::new(1, 7), CaretPos::new(1, 8)),
                                token: Token::Id(String::from("b")),
                            },
                        ]],
                    ),
                },
                Lex {
                    pos: Position::new(CaretPos::new(1, 11), CaretPos::new(1, 14)),
                    token: Token::Eof,
                },
            ]
        );
    }
}
