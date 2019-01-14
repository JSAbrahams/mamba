use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_cntrl_flow_expr(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> ParseResult<ASTNode> {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) |
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => parse_if(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::When }) => parse_when(it, ind),
        Some(&next) => Err(CustomErr {
            expected: "control flow expression".to_string(),
            actual: next.clone(),
        }),
        None => Err(CustomEOFErr { expected: "control flow expression".to_string() })
    };
}

fn parse_if(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let if_expr = match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) => true,
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => false,
        Some(next) => return Err(TokenErr { expected: Token::If, actual: next.clone() }),
        None => return Err(EOFErr { expected: Token::If })
    };

    let (cond, ind) = get_or_err!(it, ind, parse_expression, "if condition");
    check_next_is!(it, Token::Then);
    let (then_branch, ind) = get_or_err!(it, ind, parse_expr_or_stmt, "if then branch");
    if let Some(&&TokenPos { line: _, pos: _, token: Token::Else }) = it.peek() {
        it.next();
        let (else_branch, ind) = get_or_err!(it, ind, parse_expr_or_stmt, "if else branch");
        if if_expr {
            Ok((ASTNode::IfElse(cond, then_branch, else_branch), ind))
        } else { Ok((ASTNode::UnlessElse(cond, then_branch, else_branch), ind)) }
    } else {
        if if_expr {
            Ok((ASTNode::If(cond, then_branch), ind))
        } else { Ok((ASTNode::Unless(cond, then_branch), ind)) }
    }
}

fn parse_when(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, Token::When);

    let (expr, ind) = get_or_err!(it, ind, parse_expression, "when expression");
    check_next_is!(it, Token::NL);

    match parse_when_cases(it, ind + 1) {
        Ok((cases, ind)) => Ok((ASTNode::When(expr, cases), ind)),
        Err(err) => Err(err)
    }
}

fn parse_when_cases(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<Vec<ASTNode>> {
    panic!("Not implemented");
}

fn parse_when_case(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let (when, ind) = get_or_err!(it, ind, parse_expression, "when case");
    check_next_is!(it, Token::Then);
    let (then, ind) = get_or_err!(it, ind, parse_expr_or_stmt, "then");

    return Ok((ASTNode::If(when, then), ind));
}
