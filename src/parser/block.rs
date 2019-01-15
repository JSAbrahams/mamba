use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::util::ind_count;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_block(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let mut nodes = Vec::new();
    loop {
        while let Some(TokenPos { line: _, pos: _, token: Token::NL }) = it.peek() { it.next(); }

        let actual = ind_count(it);
        if actual < ind || it.peek() == None { break; }
        if actual > ind { return Err(IndErr { actual, expected: ind }); }

        let (ast_node, _) = get_or_err_direct!(it, ind, parse_expr_or_stmt, "block");
        if it.peek().is_some() { check_next_is!(it, Token::NL); }
        nodes.push(ast_node);
    }

    return Ok((ASTNode::Block(nodes), ind - 1));
}
