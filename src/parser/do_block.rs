use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// do-block         ::= { { indent } expr-or-stmt newline [ { indent } newline ] }

pub fn parse_do_block(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    let mut nodes = Vec::new();

    while let Some(_) = it.peek() {
        let actual = util::ind_count(it);
        if actual > ind && it.peek().is_some() {
            return (Err(IndErr { expected: ind, actual }), ind);
        }

        match parse_expr_or_stmt(it, ind) {
            (Ok(ast_node), ind) => nodes.push(ast_node),
            err => return err
        }

        /* empty line */
        if let Some(&&tp) = it.peek() {
            if tp.token != Token::NL { break; }
            it.next();
            if let Some(&&tp) = it.peek() {
                if tp.token != Token::NL { break; }
                it.next();
                if let Some(&&tp) = it.peek() {
                    if tp.token == Token::NL { break; }
                }
            }
        }
    }

    return (Ok(ASTNode::Do(nodes)), ind - 1);
}