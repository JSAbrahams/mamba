use std::path::PathBuf;

#[cfg(test)]
use crate::parse::ast::{AST, Node};
use crate::parse::iterator::LexIterator;
use crate::parse::lex::tokenize;
use crate::parse::result::{ParseErr, ParseResult, ParseResults};

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

pub type ParseInput = (String, Option<PathBuf>);

/// Parse input, which is a string.
pub fn parse(input: &str) -> ParseResult {
    let tokens = tokenize(input).map_err(ParseErr::from)?;
    file::parse_file(&mut LexIterator::new(tokens.iter().peekable()))
}

pub fn parse_all(inputs: &[ParseInput]) -> ParseResults {
    let results: Vec<(ParseResult, Option<String>, Option<PathBuf>)> = inputs
        .iter()
        .map(|(source, path)| (parse(source), source, path))
        .map(|(result, source, path)| {
            let result = result.map_err(|err| err.into_with_source(&Some(source.clone()), path));
            (result, Some(source.clone()), path.clone())
        })
        .collect();

    let (oks, errs): (Vec<_>, Vec<_>) = results.iter().partition(|(res, ..)| res.is_ok());
    if errs.is_empty() {
        Ok(oks
            .iter()
            .map(|(res, src, path)| (*res.as_ref().unwrap().clone(), src.clone(), path.clone()))
            .collect())
    } else {
        Err(errs.iter().map(|(res, ..)| res.as_ref().unwrap_err().clone()).collect())
    }
}

#[cfg(test)]
fn parse_direct(input: &str) -> ParseResult<Vec<AST>> {
    match parse(input)?.node {
        Node::File { statements, .. } => Ok(statements),
        _ => Ok(vec![]),
    }
}
