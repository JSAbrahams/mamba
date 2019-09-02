use std::path::PathBuf;

use crate::lexer::token::TokenPos;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::{ParseResult, ParseResults};

pub mod ast;

mod iterator;

pub mod parse_result;

mod _type;
mod block;
mod call;
mod class;
mod collection;
mod control_flow_expr;
mod control_flow_stmt;
mod definition;
mod expr_or_stmt;
mod expression;
mod file;
mod operation;
mod statement;

pub type ParseInput = (Vec<TokenPos>, Option<String>, Option<PathBuf>);

/// Parse input, which is a slice of [TokenPos](crate::lexer::token::TokenPos).
///
/// Should never panic.
///
/// # Examples
///
/// ```
/// # use mamba::lexer::token::Token;
/// # use mamba::lexer::token::TokenPos;
/// # use mamba::parser::parse;
/// // Assigning 10 to b
/// let def = TokenPos { st_line: 0, st_pos: 0, token: Token::Def };
/// let id = TokenPos { st_line: 0, st_pos: 4, token: Token::Id(String::from("b")) };
/// let assign = TokenPos { st_line: 0, st_pos: 6, token: Token::Assign };
/// let number = TokenPos { st_line: 0, st_pos: 9, token: Token::Int(String::from("9")) };
///
/// let result = parse(&[def, id, assign, number]);
/// assert_eq!(result.is_ok(), true);
/// ```
///
/// # Failures
///
/// If we receive an illegal sequence of tokens it fails.
///
/// ```
/// # use mamba::lexer::token::Token;
/// # use mamba::lexer::token::TokenPos;
/// # use mamba::parser::parse;
/// let def = TokenPos { st_line: 0, st_pos: 0, token: Token::Def };
/// let id = TokenPos { st_line: 0, st_pos: 4, token: Token::Id(String::from("b")) };
/// let number = TokenPos { st_line: 0, st_pos: 9, token: Token::Int(String::from("9")) };
///
/// let result = parse(&[def, id, number]);
/// assert_eq!(result.is_err(), true);
/// ```
pub fn parse(input: &[TokenPos]) -> ParseResult {
    file::parse_file(&mut TPIterator::new(input.iter().peekable()))
}

pub fn parse_all(inputs: &[ParseInput]) -> ParseResults {
    let inputs: Vec<_> = inputs
        .iter()
        .map(|(node_pos, source, path)| (parse(node_pos), source, path))
        .map(|(result, source, path)| {
            (result.map_err(|err| err.into_with_source(source, path)), source.clone(), path.clone())
        })
        .collect();

    let (oks, errs): (Vec<_>, Vec<_>) = inputs.iter().partition(|(res, ..)| res.is_ok());
    if errs.is_empty() {
        Ok(oks
            .iter()
            .map(|(res, src, path)| (*res.as_ref().unwrap().clone(), src.clone(), path.clone()))
            .collect())
    } else {
        Err(errs.iter().map(|(res, ..)| res.as_ref().unwrap_err().clone()).collect())
    }
}

/// Parse input as a script.
pub fn parse_direct(input: &[TokenPos]) -> ParseResult {
    file::parse_script(&mut TPIterator::new(input.iter().peekable()))
}
