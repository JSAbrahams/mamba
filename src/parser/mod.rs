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
/// // examples here
///
/// # Failures
///
/// // examples of failure here
///
pub fn parse(input: &[TokenPos]) -> ParseResult { file::parse_file(&mut input.iter().peekable()) }

/// Parse input as a script.
pub fn parse_direct(input: &[TokenPos]) -> ParseResult {
    file::parse_script(&mut input.iter().peekable())
}
