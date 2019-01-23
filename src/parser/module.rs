use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::block::parse_block_no_indent;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;

pub fn parse_module(it: &mut TPIterator) -> ParseResult {
    match parse_block_no_indent(it) {
        Ok(body) =>
            Ok(ASTNodePos {
                st_line: 0,
                st_pos: 0,
                en_line: 0,
                en_pos: 0,
                node: ASTNode::Script {
                    imports: vec![ASTNodePos {
                        st_line: 0,
                        st_pos: 0,
                        en_line: 0,
                        en_pos: 0,
                        node: ASTNode::Break,
                    }],
                    decl: Box::new(ASTNodePos {
                        st_line: 0,
                        st_pos: 0,
                        en_line: 0,
                        en_pos: 0,
                        node: ASTNode::Break,
                    }),
                    funcs: vec![ASTNodePos {
                        st_line: 0,
                        st_pos: 0,
                        en_line: 0,
                        en_pos: 0,
                        node: ASTNode::Break,
                    }],
                    body: Box::new(body),
                },
            }),
        err => err
    }
}
