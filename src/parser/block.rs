use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::util;
use crate::parser::util::detect_double_newline;
use std::iter::Peekable;
use std::slice::Iter;

// block            ::= { { indent } expr-or-stmt newline [ { indent } newline ] }

pub fn parse_do_block(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let mut nodes = Vec::new();

    while let Some(_) = it.peek() {
        let actual = util::ind_count(it);
        if actual > ind && it.peek().is_some() { return Err(IndErr { expected: ind, actual }); }
        if actual < ind {
            match it.peek() {
                /* indentation decreased, but newline, so might continue */
                Some(TokenPos { line: _, pos: _, token: Token::NL }) => next_and!(it, continue),
                _ => break
            }
        };

        let (ast_node, _) = get_or_err_direct!(it, ind, parse_expr_or_stmt, "do block");
        nodes.push(ast_node);

        if detect_double_newline(it) { break; }
    }

    return Ok((ASTNode::Do(nodes), ind - 1));
}