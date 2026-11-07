use std::str::FromStr;

use crate::common::position::Position;
use crate::parse::ast::{Node, AST};
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Lex;
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
mod lex;
mod operation;
mod statement;
mod ty;

impl FromStr for AST {
    type Err = Box<ParseErr>;

    fn from_str(input: &str) -> ParseResult<AST> {
        let tokens: Vec<Lex> = tokenize(input).map_err(ParseErr::from)?;

        let mut iterator = LexIterator::new(tokens.iter().peekable());
        let statements = block::parse_statements(&mut iterator)?;
        if let Some(next) = iterator.peek_next() {
            return Err(Box::new(crate::parse::result::custom(
                &format!("unexpected token '{}'", next.token.name()),
                next.pos,
            )));
        }

        let start = statements
            .first()
            .map_or_else(Position::invisible, |stmt| stmt.pos);
        let end = statements
            .last()
            .map_or_else(Position::invisible, |stmt| stmt.pos);

        Ok(AST::new(start.union(end), Node::Block { statements }))
    }
}

#[cfg(test)]
pub fn parse_direct(input: &str) -> ParseResult<Vec<AST>> {
    match AST::from_str(input)?.node {
        Node::Block { statements } => Ok(statements),
        _ => Ok(vec![]),
    }
}
