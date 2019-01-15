use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::block::parse_block;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;
use std::env;

pub fn parse_module(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    match parse_block(it, 0) {
        Ok((prog, _)) => Ok((ASTNode::Script(vec![ASTNode::Break], vec![ASTNode::Break],
                                             Box::new(prog)), 0)),
        err => err
    }
}
