use std::path::PathBuf;

use crate::parse::lex::lex_result::{LexResult, LexResults};
use crate::parse::lex::pass::pass;
use crate::parse::lex::state::State;
use crate::parse::lex::tokenize::into_tokens;

pub mod lex_result;
pub mod token;

#[macro_use]
mod state;
mod pass;
mod tokenize;

pub type TokenizeInput = (String, Option<PathBuf>);

/// Convert a given string to a sequence of
/// [TokenPos](mamba::lexer::token::TokenPos), each containing a
/// [Token](mamba::lexer::token::Token), in addition to line number and
/// position. Note that line number and position are 1-indexed.
///
/// Should never panic.
///
/// # Examples
///
/// ```
/// # use mamba::parse::lex::tokenize;
/// # use mamba::parse::lex::token::Token;
/// # use mamba::parse::lex::token::Lex;
/// # use mamba::common::position::CaretPos;
/// let source = "a := 2";
/// let tokens = tokenize(&source).unwrap();
///
/// assert_eq!(tokens[0].clone(), Lex::new(&CaretPos::new(1, 1), Token::Id(String::from("a"))));
/// assert_eq!(tokens[1], Lex::new(&CaretPos::new(1, 3), Token::Assign));
/// assert_eq!(tokens[2], Lex::new(&CaretPos::new(1, 6), Token::Int(String::from("2"))));
/// ```
///
/// # Failures
///
/// Fails if it encounters an unrecognized character.
///
/// ```
/// # use mamba::parse::lex::tokenize;
/// // The '$' character is meaningless in Mamba.
/// let source = "$";
/// let result = tokenize(&source);
/// assert_eq!(result.is_err(), true);
/// ```
#[allow(clippy::while_let_on_iterator)]
pub fn tokenize(input: &str) -> LexResult {
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

pub fn tokenize_all(inputs: &[TokenizeInput]) -> LexResults {
    let inputs: Vec<_> = inputs
        .iter()
        .map(|(source, path)| (tokenize(source), source, path))
        .map(|(result, source, path)| {
            let result = result.map_err(|err| err.into_with_source(&Some(source.clone()), path));
            (result, Some(source.clone()), path.clone())
        })
        .collect();

    let (oks, errs): (Vec<_>, Vec<_>) = inputs.iter().partition(|(res, ..)| res.is_ok());
    if errs.is_empty() {
        Ok(oks.into_iter().cloned().map(|(res, src, path)| (res.unwrap(), src, path)).collect())
    } else {
        Err(errs.iter().map(|(res, ..)| res.as_ref().unwrap_err().clone()).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::position::{CaretPos, Position};
    use crate::parse::lex::token::{Lex, Token};
    use crate::parse::lex::tokenize;

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
}