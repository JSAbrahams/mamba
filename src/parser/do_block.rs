use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseError;
use crate::parser::parse_result::ParseResult;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// do-block         ::= { { indent } expr-or-stmt newline [ { indent } newline ] }

pub fn parse_do_block(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    let mut nodes = Vec::new();

    while let Some(_) = it.peek() {
        let next_ind = util::ind_count(it);
        if next_ind > ind && it.peek().is_some() {
            return (Err(format!("Expected indentation of {}.", ind)), next_ind);
        }

        match parse_expr_or_stmt(it, ind) {
            (Ok(ast_node), ind) => {
                match it.next() {
                    Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::Then =>
                        return (Err(ParseError::TokenError(*tp, Token::Then)), ind)
                }
                nodes.push(ast_node)
            }
            err => return err
        }

        /* empty line */
        if Some(&&TokenPos { line, pos, token: Token::NL }) == it.peek() {
            it.next();
            if Some(&&TokenPos { line, pos, token: Token::NL }) == it.peek() {
                it.next();
                if Some(&&TokenPos { line, pos, token: Token::NL }) == it.peek() { break; }
            }
        }
    }

    return (Ok(ASTNode::Do(nodes)), ind - 1);
}