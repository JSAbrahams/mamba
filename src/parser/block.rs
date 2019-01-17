use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;
use std::env;

pub fn parse_block(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let mut stmts = Vec::new();
    let mut en_line = None;
    let mut en_pos = None;
    loop {
        if it.peek().is_none() || it.peek().unwrap().token == Token::Dedent { break; }
        let ast_node: ASTNodePos = get_or_err_direct!(it, parse_expr_or_stmt, "block");
        stmts.push(ast_node);

        en_line = ast_node.en_line;
        en_pos = ast_node.en_pos;
    }

    if it.peek().is_some() { check_next_is!(it, Token::Dedent); }
    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Block { stmts } });
}
