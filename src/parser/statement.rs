use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::declaration::parse_declaration;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use std::env;

pub fn parse_statement(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    print_parse!(it, ind, "statement");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Print }) => {
            it.next();
            let (expr, ind) = get_or_err!(it, ind, parse_expression, "statement");
            Ok((ASTNode::Print(expr), ind))
        }

        Some(TokenPos { line: _, pos: _, token: Token::Let }) |
        Some(TokenPos { line: _, pos: _, token: Token::Mut }) => parse_declaration(it, ind),

        Some(TokenPos { line: _, pos: _, token: Token::For }) |
        Some(TokenPos { line: _, pos: _, token: Token::While }) => parse_cntrl_flow_stmt(it, ind),

        Some(&next) => Err(CustomErr { expected: "statement".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "statement".to_string() })
    };
}
