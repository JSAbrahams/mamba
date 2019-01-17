use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;

pub fn parse_module(it: &mut TPIterator) -> ParseResult {
    match parse_block(it) {
        Ok(body) =>
            Ok(ASTNodePos {
                st_line: None,
                st_pos: None,
                en_line: None,
                en_pos: None,
                node: ASTNode::Script {
                    imports: vec![ASTNodePos {
                        st_line: None,
                        st_pos: None,
                        en_line: None,
                        en_pos: None,
                        node: ASTNode::Break,
                    }],
                    decl: Box::new(ASTNodePos {
                        st_line: None,
                        st_pos: None,
                        en_line: None,
                        en_pos: None,
                        node: ASTNode::Break,
                    }),
                    funcs: vec![ASTNodePos {
                        st_line: None,
                        st_pos: None,
                        en_line: None,
                        en_pos: None,
                        node: ASTNode::Break,
                    }],
                    body: Box::new(body),
                },
            }),
        err => err
    }
}
