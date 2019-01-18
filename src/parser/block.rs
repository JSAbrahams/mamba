use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_block(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let mut stmts = Vec::new();
    let mut en_line = None;
    let mut en_pos = None;
    loop {
        match it.peek() {
            None | Some(TokenPos { token: Token::Dedent, .. }) => break,
            Some(TokenPos { token: Token::NL, .. }) => {
                it.next();
                continue;
            },
            _ => {
                let ast_node: ASTNodePos = get_or_err_direct!(it, parse_expr_or_stmt, "block");
                en_line = ast_node.en_line;
                en_pos = ast_node.en_pos;

                stmts.push(ast_node);
            }
        }
    }

    if it.peek().is_some() { check_next_is!(it, Token::Dedent); }
    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Block { stmts } });
}
