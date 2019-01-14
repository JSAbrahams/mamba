use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::util;
use crate::parser::util::ind_count;
use std::iter::Peekable;
use std::slice::Iter;

// block            ::= { { indent } expr-or-stmt newline { newline } }

pub fn parse_block(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let mut nodes = Vec::new();
    while in_block(it, ind) {
        let (ast_node, _) = get_or_err_direct!(it, ind, parse_expr_or_stmt, "block");
        nodes.push(ast_node);
    }

    return Ok((ASTNode::Do(nodes), ind - 1));
}

fn in_block(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> bool {
    while let Some(TokenPos { line: _, pos: _, token: Token::NL }) = it.peek() { it.next(); }
    return ind_count(it) == ind && it.peek() != None;
}
