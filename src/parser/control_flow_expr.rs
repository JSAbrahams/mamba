use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::function::parse_function_anonymous;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-expr::= if | from | when
// if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
// from             ::= "from" maybe-expr [ newline ] "where" function-anon  [ "map" function-anon ]
// when             ::= "when" maybe-expr newline { { indent } when-case }
// when-case        ::= maybe-expr "then" expr-or-stmt

pub fn parse_cntrl_flow_expr(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) |
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => parse_if(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::From }) => parse_from(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::When }) => parse_when(it, ind),
        Some(&next) => (Err(TokenErr { expected: Token::If, actual: next.clone() }), ind),
        None => (Err(EOFErr { expected: Token::If }), ind)
    };
}

fn parse_if(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    let if_expr = match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) => true,
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) => false,
        Some(next) => return (Err(TokenErr { expected: Token::If, actual: next.clone() }), ind),
        None => return (Err(EOFErr { expected: Token::If }), ind)
    };

    return match parse_expression(it, ind) {
        (Ok(cond), ind) => {
            check_next_is!(it, ind, Token::Then);
            match parse_expr_or_stmt(it, ind) {
                (Ok(then), ind) =>
                    if let Some(&&TokenPos { line: _, pos: _, token: Token::Else }) = it.peek() {
                        it.next();
                        match parse_expr_or_stmt(it, ind) {
                            (Ok(otherwise), ind) => if if_expr {
                                (Ok(ASTNode::IfElse(wrap!(cond), wrap!(then), wrap!(otherwise))),
                                 ind)
                            } else {
                                (Ok(ASTNode::UnlessElse(wrap!(cond), wrap!(then), wrap!(otherwise))),
                                 ind)
                            }
                            err => err
                        }
                    } else {
                        if if_expr {
                            (Ok(ASTNode::If(wrap!(cond), wrap!(then))), ind)
                        } else {
                            (Ok(ASTNode::Unless(wrap!(cond), wrap!(then))), ind)
                        }
                    }
                err => err
            }
        }
        err => err
    };
}

fn parse_from(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::From);
    return match parse_expression(it, ind) {
        (Ok(coll), ind) => {
            check_next_is!(it, ind, Token::When);
            match parse_function_anonymous(it, ind) {
                (Ok(cond), ind) => match it.peek() {
                    Some(TokenPos { line: _, pos: _, token: Token::Map }) =>
                        match (it.next(), parse_function_anonymous(it, ind)) {
                            (_, (Ok(mapping), ind)) => (Ok(
                                ASTNode::FromMap(wrap!(coll), wrap!(cond), wrap!(mapping))), ind),
                            (_, err) => err
                        }
                    _ => (Ok(ASTNode::From(wrap!(coll), wrap!(cond))), ind)
                }
                err => err
            }
        }
        err => err
    };
}

fn parse_when(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::When);
    match parse_expression(it, ind) {
        (Ok(expr), ind) => {
            check_next_is!(it, ind, Token::NL);
            match parse_when_cases(it, ind + 1) {
                (Ok(cases), ind) => (Ok(ASTNode::When(wrap!(expr), cases)), ind),
                (Err(err), ind) => (Err(err), ind)
            }
        }
        err => err
    }
}

fn parse_when_cases(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                    -> (ParseResult<Vec<ASTNode>>, i32) {
    let mut when_cases = Vec::new();

    while let Some(_) = it.peek() {
        let next_ind = util::ind_count(it);
        if next_ind < ind { break; }; /* Indentation decrease marks end of do block */
        if next_ind > ind && it.peek().is_some() {
            return (Err(IndErr { expected: ind, actual: next_ind }), next_ind);
        }

        match parse_when_case(it, ind) {
            (Err(err), ind) => return (Err(err), ind),
            (Ok(case), _) => when_cases.push(case),
        }

        /* empty line */
        if let Some(&tp) = it.peek() {
            if tp.token != Token::NL { break; }
            it.next();
            if let Some(&tp) = it.peek() {
                if tp.token != Token::NL { break; }
                it.next();
                if let Some(&tp) = it.peek() {
                    if tp.token == Token::NL { break; }
                }
            }
        }
    }

    return (Ok(when_cases), ind);
}

fn parse_when_case(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match parse_expression(it, ind) {
        (Ok(expr), ind) => {
            check_next_is!(it, ind, Token::Then);
            match parse_expr_or_stmt(it, ind) {
                (Ok(expr_or_do), ind) => (Ok(ASTNode::If(wrap!(expr), wrap!(expr_or_do))), ind),
                err => err
            }
        }
        err => err
    }
}
