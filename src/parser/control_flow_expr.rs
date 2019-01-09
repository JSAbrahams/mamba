use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-expr::= if | when | from
// if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
// from             ::= "from" maybe-expr [ newline ] "where" maybe-expression
// when             ::= "when" maybe-expr newline { { indent } when-case }
// when-case        ::= maybe-expr "then" expr-or-stmt

pub fn parse_cntrl_flow_expr(it: &mut Peekable<Iter<Token>>, ind: i32)
                             -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::If) => parse_if(it, ind),
        Some(Token::Unless) => parse_unless(it, ind),
        Some(Token::When) => parse_when(it, ind),
        Some(Token::From) => panic!("Not implemented"),

        Some(_) | None => panic!("Expected control flow expression.")
    };
}

fn parse_unless(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Unless));

    return match parse_expression(it, ind) {
        (Ok(cond), ind) => if it.next() != Some(&Token::Then) {
            return (Err("'Then' keyword expected".to_string()), ind);
        } else {
            match parse_expr_or_stmt(it, ind) {
                (Ok(then), ind) => if Some(&&Token::Else) != it.peek() {
                    (Ok(ASTNode::Unless(wrap!(cond), wrap!(then))), ind)
                } else {
                    it.next();
                    match parse_expr_or_stmt(it, ind) {
                        (Ok(otherwise), ind) => (Ok(ASTNode::UnlessElse(
                            wrap!(cond), wrap!(then), wrap!(otherwise))), ind),
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}

fn parse_if(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::If));

    return match parse_expression(it, ind) {
        (Ok(cond), ind) => if it.next() != Some(&Token::Then) {
            return (Err("'Then' keyword expected".to_string()), ind);
        } else {
            match parse_expr_or_stmt(it, ind) {
                (Ok(then), ind) => if Some(&&Token::Else) != it.peek() {
                    (Ok(ASTNode::If(wrap!(cond), wrap!(then))), ind)
                } else {
                    it.next();
                    match parse_expr_or_stmt(it, ind) {
                        (Ok(otherwise), ind) => (Ok(ASTNode::IfElse(
                            wrap!(cond), wrap!(then), wrap!(otherwise))), ind),
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}

fn parse_when(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::When));

    match parse_expression(it, ind) {
        (Ok(expr), ind) => if it.next() != Some(&Token::NL) {
            (Err("Expected newline after 'is' in 'when' expression".to_string()), ind)
        } else {
            match parse_when_cases(it, ind + 1) {
                (Ok(cases), ind) => (Ok(ASTNode::When(wrap!(expr), cases)), ind),
                (Err(err), ind) => (Err(err), ind)
            }
        }
        err => err
    }
}

fn parse_when_cases(it: &mut Peekable<Iter<Token>>, ind: i32)
                    -> (Result<Vec<ASTNode>, String>, i32) {
    let mut when_cases = Vec::new();

    while let Some(_) = it.peek() {
        let next_ind = util::ind_count(it);
        if next_ind < ind { break; }; /* Indentation decrease marks end of do block */
        if next_ind > ind && it.peek().is_some() {
            return (Err(format!("Expected indentation of {}.", ind)), next_ind);
        }

        match parse_when_case(it, ind) {
            (Err(err), ind) => return (Err(err), ind),
            (Ok(case), _) => when_cases.push(case),
        }

        if Some(&&Token::NL) == it.peek() {
            it.next();
            if Some(&&Token::NL) == it.peek() {
                it.next();
                break;
            }
        } else { break; }
    }

    return (Ok(when_cases), ind);
}

fn parse_when_case(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match parse_expression(it, ind) {
        (Ok(expr), ind) => if it.next() != Some(&Token::Then) {
            return (Err("Expected 'then' after when case expression".to_string()), ind);
        } else {
            match parse_expr_or_stmt(it, ind) {
                (Ok(expr_or_do), ind) => (Ok(ASTNode::If(wrap!(expr), wrap!(expr_or_do))), ind),
                err => err
            }
        }
        err => err
    }
}
