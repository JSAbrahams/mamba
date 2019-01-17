use crate::parser::ASTNode;
use crate::parser::block::parse_block;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;

pub fn parse_module(it: &mut TPIterator) -> ParseResult {
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
