use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::declaration::parse_declaration;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::env;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_statement(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it, "statement");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Print }) => {
            it.next();
            let expr = get_or_err!(it, parse_expression, "statement");
            Ok(ASTNode::Print { expr })
        }

        Some(TokenPos { line: _, pos: _, token: Token::Let }) |
        Some(TokenPos { line: _, pos: _, token: Token::Mut }) => parse_declaration(it),

        Some(TokenPos { line: _, pos: _, token: Token::For }) |
        Some(TokenPos { line: _, pos: _, token: Token::While }) => parse_cntrl_flow_stmt(it),

        Some(&next) => Err(CustomErr { expected: "statement".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "statement".to_string() })
    };
}
