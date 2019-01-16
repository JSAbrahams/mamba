use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::block::parse_block;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_module(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    match parse_block(it) {
        Ok(body) =>
            Ok(ASTNode::Script {
                imports: vec![ASTNode::Break],
                decl: Box::new(ASTNode::Break),
                funcs: vec![ASTNode::Break],
                body: Box::new(body),
            }),
        err => err
    }
}
