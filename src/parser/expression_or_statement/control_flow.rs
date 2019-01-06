use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse_maybe_expression as parse_maybe_expression;
use crate::parser::expression_or_statement::parse as parse_expr_or_stmt;
use crate::parser::util::ind_count;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-expr ::= if | when
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::If) => parse_if(it, ind),
        Some(Token::Unless) => parse_unless(it, ind),
        Some(Token::When) => parse_when(it, ind),

        Some(t) => panic!(format!("Expected control flow expression, but other token: {:?}", t)),
        None => panic!("Expected control flow expression, but end of file.")
    };
}

// if ::= ( [...] | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
fn parse_unless(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Unless));

    return match parse_maybe_expression(it, ind) {
        (Ok(cond), new_ind) => {
            if it.next() != Some(&Token::Then) {
                return (Err("'Then' keyword expected".to_string()), new_ind);
            }

            match parse_expr_or_stmt(it, new_ind) {
                (Ok(then), nnew_ind) => match it.peek() {
                    Some(Token::Else) => {
                        it.next();
                        match parse_expr_or_stmt(it, nnew_ind) {
                            (Ok(otherwise), nnnew_ind) => (Ok(ASTNode::UnlessElse(
                                Box::new(cond),
                                Box::new(then),
                                Box::new(otherwise))), nnnew_ind),
                            err => err
                        }
                    }
                    _ => (Ok(ASTNode::Unless(Box::new(cond), Box::new(then))), nnew_ind)
                }
                err => err
            }
        }
        err => err
    };
}

// if ::= ( "if" | [...] ) expression "then" expr-or-stmt-or-do [ "else" expr-or-stmt-or-do ]
fn parse_if(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::If));

    return match parse_maybe_expression(it, ind) {
        (Ok(cond), new_ind) => {
            if it.next() != Some(&Token::Then) {
                return (Err("'Then' keyword expected".to_string()), new_ind);
            }

            match parse_expr_or_stmt(it, new_ind) {
                (Ok(then), nnew_ind) => match it.peek() {
                    Some(Token::Else) => {
                        it.next();
                        match parse_expr_or_stmt(it, nnew_ind) {
                            (Ok(otherwise), nnnew_ind) => (Ok(ASTNode::IfElse(
                                Box::new(cond),
                                Box::new(then),
                                Box::new(otherwise))), nnnew_ind),
                            err => err
                        }
                    }
                    _ => (Ok(ASTNode::If(Box::new(cond), Box::new(then))), nnew_ind)
                }
                err => err
            }
        }
        err => err
    };
}

// when ::= "when" maybe-expr "is" newline { { indent } when-case }
fn parse_when(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::When));

    match parse_maybe_expression(it, ind) {
        (Ok(expr), new_ind) => {
            if it.next() != Some(&Token::Is) {
                return (Err("Expected 'is' after 'when' expression".to_string()), new_ind);
            } else if it.next() != Some(&Token::NL) {
                return (Err("Expected newline after 'is' in 'when' expression".to_string()),
                        new_ind);
            }

            match parse_when_cases(it, ind + 1) {
                (Ok(cases), _) => (Ok(ASTNode::When(Box::new(expr), cases)), ind),
                (Err(err), new_ind) => (Err(err), new_ind)
            }
        }
        err => err
    }
}

fn parse_when_cases(it: &mut Peekable<Iter<Token>>, ind: i32)
                    -> (Result<Vec<ASTNode>, String>, i32) {
    let act_ind = ind_count(it);
    if ind != act_ind {
        return (Err(format!("Expected indentation level {}, was {}.", ind, act_ind)), act_ind);
    }

    let mut when_cases = Vec::new();
    let mut is_prev_empty_line = false;

    loop {
        match it.peek() {
            /* double empty line marks end of when */
            Some(Token::NL) if is_prev_empty_line => break,
            Some(Token::NL) => {
                is_prev_empty_line = true;
                it.next();
                continue;
            }
            _ => ()
        }

        let (res, this_ind) = parse_when_case(it, ind);
        if it.next() != Some(&Token::NL) {
            return (Err("Expected newline after 'when' case expression".to_string()), ind);
        }

        let next_ind = ind_count(it);
        if next_ind < ind { break; }; /* Indentation decrease marks end of do when */
        if next_ind > ind && it.peek().is_some() {
            /* indentation increased unexpectedly */
            return (Err(format!("Indentation increased in do block from {} to {}.", ind, next_ind)),
                    ind);
        }

        match res {
            Ok(when_case) => when_cases.push(when_case),
            Err(err) => return (Err(err), this_ind)
        }
    }

    (Ok(when_cases), ind)
}

// when-case ::= maybe-expr "then" expr-or-stmt
fn parse_when_case(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match parse_maybe_expression(it, ind) {
        (Ok(expr), new_ind) => {
            if it.next() != Some(&Token::Then) {
                return (Err("Expected 'then' after when case expression".to_string()), new_ind);
            }

            match parse_expr_or_stmt(it, ind) {
                (Ok(expr_or_do), nnew_ind) =>
                    (Ok(ASTNode::If(Box::new(expr), Box::new(expr_or_do))), nnew_ind),
                err => err
            }
        }
        err => err
    }
}
