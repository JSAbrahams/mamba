use std::path::PathBuf;

use crate::lex::lex_result::{LexResult, LexResults};
use crate::lex::pass::pass;
use crate::lex::state::State;
use crate::lex::tokenize::into_tokens;

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
/// # use mamba::lex::tokenize;
/// # use mamba::lex::token::Token;
/// # use mamba::lex::token::Lex;
/// # use mamba::common::position::CaretPos;
/// let source = "a <- 2";
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
/// # use mamba::lex::tokenize;
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
