use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;
use std::env;

pub fn parse_block(it: &mut TPIterator) -> ParseResult {
    print_parse!(it, "do block");

    let mut stmts = Vec::new();
    loop {
        if it.peek().is_none() || it.peek().unwrap().token == Token::Dedent { break; }
        let ast_node = get_or_err_direct!(it, parse_expr_or_stmt, "block");
        stmts.push(ast_node);
    }

    if it.peek().is_some() { check_next_is!(it, Token::Dedent); }
    return Ok(ASTNode::Block { stmts });
}
