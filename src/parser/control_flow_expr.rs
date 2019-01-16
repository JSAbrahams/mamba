use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::env;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_cntrl_flow_expr(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it,  "control flow expression");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) |
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => parse_if(it),
        Some(TokenPos { line: _, pos: _, token: Token::When }) => parse_when(it),
        Some(&next) => Err(CustomErr {
            expected: "control flow expression".to_string(),
            actual: next.clone(),
        }),
        None => Err(CustomEOFErr { expected: "control flow expression".to_string() })
    };
}

fn parse_if(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it, "if");

    let if_expr = match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) => true,
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => false,
        Some(next) => return Err(TokenErr { expected: Token::If, actual: next.clone() }),
        None => return Err(EOFErr { expected: Token::If })
    };

    let cond = get_or_err!(it, parse_expression, "if condition");
    check_next_is!(it, Token::Then);
    print_parse!(it, "if: then");
    let then = get_or_err!(it, parse_expr_or_stmt, "if then branch");
    if let Some(&&TokenPos { line: _, pos: _, token: Token::Else }) = it.peek() {
        print_parse!(it, "if: else");
        it.next();
        let _else = get_or_err!(it, parse_expr_or_stmt, "if else branch");
        if if_expr {
            Ok(ASTNode::IfElse { cond, then, _else })
        } else { Ok(ASTNode::UnlessElse { cond, then, _else }) }
    } else {
        if if_expr {
            Ok(ASTNode::If { cond, then })
        } else { Ok(ASTNode::Unless { cond, then }) }
    }
}

fn parse_when(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it, "when");
    check_next_is!(it, Token::When);

    let cond = get_or_err!(it, parse_expression, "when expression");
    check_next_is!(it, Token::NL);

    panic!("not implemented")
}

fn parse_when_case(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult<ASTNode> {
    print_parse!(it, "when case");

    let cond = get_or_err!(it, parse_expression, "when case");
    check_next_is!(it, Token::Then);
    let then = get_or_err!(it, parse_expr_or_stmt, "then");

    return Ok(ASTNode::If { cond, then });
}
