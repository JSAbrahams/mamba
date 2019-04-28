use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;

#[macro_use]
/// Evaluates the result.
///
/// If it is an Ok tuple, return Boxed [`ASTNodePos`].
/// If it is error, return [`ParseErr`] with the [`Err`] wrapped.
macro_rules! get_or_err {
    ($it:expr, $fun:path, $msg:expr) => {{
        let current = $it.peek().cloned();
        match $fun($it) {
            Ok(node) => Box::new(node),
            Err(err) =>
                return match current {
                    Some(tp) => Err(ParseErr {
                        parsing:  $msg.to_string(),
                        cause:    Box::new(err),
                        position: Some(tp.clone())
                    }),
                    None => Err(ParseErr {
                        parsing:  $msg.to_string(),
                        cause:    Box::new(err),
                        position: None
                    })
                },
        }
    }};
}

/// Evaluates the expression and check result.
///
/// If it is an [`Ok`] tuple, return [`ASTNodePos`]..
/// If it is error, return [`ParseErr`] with [`Err`] wrapped.
macro_rules! get_or_err_direct {
    ($it:expr, $fun:path, $msg:expr) => {{
        let current = $it.peek().cloned();
        match $fun($it) {
            Ok(node) => node,
            Err(e) =>
                return match current {
                    Some(tp) => Err(ParseErr {
                        parsing:  $msg.to_string(),
                        cause:    Box::new(e),
                        position: Some(tp.clone())
                    }),
                    None => Err(ParseErr {
                        parsing:  $msg.to_string(),
                        cause:    Box::new(e),
                        position: None
                    })
                },
        }
    }};
}

/// Check that the next is of expected token type.
///
/// If it is not of the expected token type, returns [`TokenErr`].
/// If there is no token ([`iterator::next()`] returns [`None`]), returns
/// [`EOFErr`].
macro_rules! check_next_is {
    ($it:expr, $tok:path) => {
        if let Some(next) = $it.next() {
            if next.token != $tok {
                return Err(TokenErr { expected: $tok, actual: next.clone() });
            }
            next.clone()
        } else {
            return Err(EOFErr { expected: $tok });
        }
    };
}

type TPIterator<'a> = Peekable<Iter<'a, TokenPos>>;

/// Gets the starting line and position of the current token using
/// [`TPIterator.peek()`].
fn start_pos(it: &mut TPIterator) -> (i32, i32) {
    match it.peek() {
        Some(TokenPos { line, pos, .. }) => (*line, *pos),
        None => (0, 0)
    }
}

/// Gets the end line and position of the current token using
/// [`TPIterator.peek()`].
///
/// The end position is calculated by offsetting the starting position by the
/// offset of the current token, by calling its [`fmt::Display`] method.
fn end_pos(it: &mut TPIterator) -> (i32, i32) {
    match it.peek() {
        Some(TokenPos { line, pos, token }) => {
            let tok_width = match token {
                Token::Id(id) => id.len(),
                Token::Real(real) => real.len(),
                Token::Int(int) => int.len(),
                Token::Bool(true) => 4,
                Token::Bool(false) => 5,
                Token::Str(_str) => _str.len(),
                Token::ENum(num, exp) => num.len() + 1 + exp.len(),
                other => format!("{}", other).len()
            } as i32;

            (*line, *pos + tok_width)
        }
        None => (-1, -1)
    }
}

pub mod ast;

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

/// Parse input as regular file.
pub fn parse(input: &[TokenPos]) -> ParseResult { file::parse_file(&mut input.iter().peekable()) }

/// Parse input as a script.
pub fn parse_direct(input: &[TokenPos]) -> ParseResult {
    file::parse_script(&mut input.iter().peekable())
}
