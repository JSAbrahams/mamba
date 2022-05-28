#[cfg(test)]
use crate::parse::ast::{AST, Node};
use crate::parse::iterator::LexIterator;
use crate::parse::lex::tokenize;
use crate::parse::result::{ParseErr, ParseResult};

pub mod ast;

mod iterator;

pub mod result;

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
mod lex;
mod operation;
mod statement;
mod ty;

/// Parse input, which is a string.
pub fn parse(input: &str) -> ParseResult {
    let tokens = tokenize(input).map_err(ParseErr::from)?;
    file::parse_file(&mut LexIterator::new(tokens.iter().peekable()))
}

#[cfg(test)]
pub fn parse_direct(input: &str) -> ParseResult<Vec<AST>> {
    match parse(input)?.node {
        Node::File { statements, .. } => Ok(statements),
        _ => Ok(vec![]),
    }
}
