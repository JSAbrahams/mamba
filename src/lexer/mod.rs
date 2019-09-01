use crate::lexer::common::State;
use crate::lexer::lex_result::LexResult;
use crate::lexer::tokenize::into_tokens;
use std::path::PathBuf;

pub mod lex_result;
pub mod token;

#[macro_use]
mod common;
mod tokenize;

/// Convert a given string to a sequence of
/// [TokenPos](crate::lexer::token::TokenPos), each containing a
/// [Token](crate::lexer::token::Token), in addition to line number and
/// position. Note that line number and position are 1-indexed.
///
/// Should never panic.
///
/// # Examples
///
/// ```
/// # use mamba::lexer::tokenize;
/// # use mamba::lexer::token::Token;
/// # use mamba::lexer::token::TokenPos;
/// let source = "a <- 2";
/// let tokens = tokenize(&source).unwrap();
///
/// assert_eq!(tokens[0], TokenPos {
///     st_line: 1,
///     st_pos:  1,
///     token:   Token::Id(String::from("a"))
/// });
/// assert_eq!(tokens[1], TokenPos { st_line: 1, st_pos: 3, token: Token::Assign });
/// assert_eq!(tokens[2], TokenPos {
///     st_line: 1,
///     st_pos:  6,
///     token:   Token::Int(String::from("2"))
/// });
/// ```
///
/// # Failures
///
/// Fails if it encounters an unrecognized character.
///
/// ```
/// # use mamba::lexer::tokenize;
/// // The '$' character on its own is meaningless in Mamba.
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

    Ok(tokens)
}

pub fn tokenize_all(inputs: &[(String, Option<String>, Option<PathBuf>)]) -> Vec<LexResult> {
    inputs
        .iter()
        .map(|(node_pos, source, path)| (tokenize(node_pos), source, path))
        .map(|(result, source, path)| result.map_err(|err| err.into_with_source(source, path)))
        .collect()
}
