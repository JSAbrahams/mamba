use crate::lexer::token::TokenPos;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseResult;

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
