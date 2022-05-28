use crate::common::position::Position;
use crate::parse::ast::{AST, Node};
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::lex::tokenize;
use crate::parse::result::{expected, ParseErr, ParseResult};

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
mod lex;
mod operation;
mod statement;
mod ty;

/// Parse input, which is a string.
pub fn parse(input: &str) -> ParseResult {
    let tokens = tokenize(input).map_err(ParseErr::from)?;
    let mut iterator = LexIterator::new(tokens.iter().peekable());
    let statements = block::parse_statements(&mut iterator)?;
    if iterator.peek_if(&|lex| lex.token != Token::Eof) {
        if let Some(lex) = iterator.peek_next() {
            return Err(expected(&Token::Eof, &lex, "end of file"));
        }
    }

    let start = statements.first().map_or_else(Position::default, |stmt| stmt.pos);
    let end = statements.last().map_or_else(Position::default, |stmt| stmt.pos);

    Ok(Box::from(AST::new(start.union(end), Node::Block { statements })))
}

#[cfg(test)]
pub fn parse_direct(input: &str) -> ParseResult<Vec<AST>> {
    match parse(input)?.node {
        Node::Block { statements } => Ok(statements),
        _ => Ok(vec![]),
    }
}
