use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::env;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_block(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    print_parse!(it, ind, "do block");

    let mut nodes = Vec::new();
    loop {
        if it.peek().is_none() || it.peek().unwrap().token == Token::Dedent { break; }
        let (ast_node, _) = get_or_err_direct!(it, ind, parse_expr_or_stmt, "block");
        nodes.push(ast_node);
    }

    if it.peek().is_some() { check_next_is!(it, Token::Dedent); }
    return Ok((ASTNode::Block(nodes), ind - 1));
}
