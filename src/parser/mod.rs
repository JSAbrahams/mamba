use crate::lexer::token::TokenPos;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;

pub mod ast;

#[macro_use]
mod common;
pub mod parse_result;

mod _type;
mod block;
mod call;
mod collection;
mod control_flow_expr;
mod control_flow_stmt;
mod definition;
mod expr_or_stmt;
mod expression;
mod file;
mod operation;
mod statement;

type TPIterator<'a> = Peekable<Iter<'a, TokenPos>>;

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
/// let def = TokenPos { line: 0, pos: 0, token: Token::Def };
/// let id = TokenPos { line: 0, pos: 4, token: Token::Id(String::from("b")) };
/// let assign = TokenPos { line: 0, pos: 6, token: Token::Assign };
/// let number = TokenPos { line: 0, pos: 9, token: Token::Int(String::from("9")) };
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
/// let def = TokenPos { line: 0, pos: 0, token: Token::Def };
/// let id = TokenPos { line: 0, pos: 4, token: Token::Id(String::from("b")) };
/// let number = TokenPos { line: 0, pos: 9, token: Token::Int(String::from("9")) };
///
/// let result = parse(&[def, id, number]);
/// assert_eq!(result.is_err(), true);
/// ```
pub fn parse(input: &[TokenPos]) -> ParseResult { file::parse_file(&mut input.iter().peekable()) }

/// Parse input as a script.
pub fn parse_direct(input: &[TokenPos]) -> ParseResult {
    file::parse_script(&mut input.iter().peekable())
}
