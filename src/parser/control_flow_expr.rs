use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::function::parse_function_anonymous;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::util;
use crate::parser::util::detect_double_newline;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-expr::= if | from | when
// if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
// from             ::= "from" maybe-expr [ newline ] "where" function-anon  [ "map" function-anon ]
// when             ::= "when" maybe-expr newline { { indent } when-case }
// when-case        ::= maybe-expr "then" expr-or-stmt

pub fn parse_cntrl_flow_expr(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> ParseResult<ASTNode> {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) |
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => parse_if(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::From }) => parse_from(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::When }) => parse_when(it, ind),
        Some(&next) => Err(TokenErr { expected: Token::If, actual: next.clone() }),
        None => Err(EOFErr { expected: Token::If })
    };
}

fn parse_if(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let if_expr = match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) => true,
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => false,
        Some(next) => return Err(TokenErr { expected: Token::If, actual: next.clone() }),
        None => return Err(EOFErr { expected: Token::If })
    };

    let (cond, ind) = get_or_err!(parse_expression(it, ind), "if condition");
    check_next_is!(it, ind, Token::Then);
    let (then_branch, ind) = get_or_err!(parse_expr_or_stmt(it, ind), "if then branch");
    if let Some(&&TokenPos { line: _, pos: _, token: Token::Else }) = it.peek() {
        it.next();
        let (else_branch, ind) = get_or_err!(parse_expr_or_stmt(it, ind), "if else branch");
        if if_expr {
            Ok((ASTNode::IfElse(cond, then_branch, else_branch), ind))
        } else { Ok((ASTNode::UnlessElse(cond, then_branch, else_branch), ind)) }
    } else {
        if if_expr {
            Ok((ASTNode::If(cond, then_branch), ind))
        } else { Ok((ASTNode::Unless(cond, then_branch), ind)) }
    }
}

fn parse_from(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, ind, Token::From);

    let (collection, ind) = get_or_err!(parse_expression(it, ind), "from collection");
    check_next_is!(it, ind, Token::When);
    let (condition, ind) = get_or_err!(parse_function_anonymous(it, ind), "from condition");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Map }) => {
            it.next();
            let (mapping, ind) = get_or_err!(parse_function_anonymous(it, ind), "from mapping");
            Ok((ASTNode::FromMap(collection, condition, mapping), ind))
        }
        _ => Ok((ASTNode::From(collection, condition), ind))
    };
}

fn parse_when(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, ind, Token::When);

    let (expr, ind) = get_or_err!(parse_expression(it, ind), "when expression");
    check_next_is!(it, ind, Token::NL);

    match parse_when_cases(it, ind + 1) {
        Ok((cases, ind)) => Ok((ASTNode::When(expr, cases), ind)),
        Err(err) => Err(err)
    }
}

fn parse_when_cases(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<Vec<ASTNode>> {
    let mut when_cases: Vec<ASTNode> = Vec::new();

    while let Some(_) = it.peek() {
        let next_ind = util::ind_count(it);
        if next_ind < ind { break; }; /* Indentation decrease marks end of do block */
        if next_ind > ind && it.peek().is_some() {
            return Err(IndErr { expected: ind, actual: next_ind });
        }

        let (when_case, ind) = get_or_err_direct!(parse_when_case(it, ind), "when case");
        when_cases.push(when_case);

        if detect_double_newline(it) { break; }
    }

    return Ok((when_cases, ind));
}

fn parse_when_case(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let (when, ind) = get_or_err!(parse_expression(it, ind), "when case");
    check_next_is!(it, ind, Token::Then);
    let (then, ind) = get_or_err!(parse_expr_or_stmt(it, ind), "then");

    return Ok((ASTNode::If(when, then), ind));
}
