use mamba::lex::token::Lex;
use mamba::parse::ast::{AST, Node};
use mamba::parse::parse;
use mamba::parse::result::ParseResult;

pub fn parse_direct(input: &[Lex]) -> ParseResult<Vec<AST>> {
    match parse(input)?.node {
         Node::File { statements } => Ok(statements),
        _ => Ok(vec![])
    }
}
