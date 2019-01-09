use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::function::parse_function_anonymous;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-expr::= if | from | when
// if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
// from             ::= "from" maybe-expr [ newline ] "where" maybe-expression
//                      [ "map" function-anon ]
// when             ::= "when" maybe-expr newline { { indent } when-case }
// when-case        ::= maybe-expr "then" expr-or-stmt

pub fn parse_cntrl_flow_expr(it: &mut Peekable<Iter<Token>>, ind: i32)
                             -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::If) | Some(Token::Unless) => parse_if(it, ind),
        Some(Token::From) => parse_from(it, ind),
        Some(Token::When) => parse_when(it, ind),

        Some(_) | None => (Err("Expected control flow expression.".to_string()), ind)
    };
}

fn parse_if(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    let if_expr = match it.next() {
        Some(Token::If) => true,
        Some(Token::Unless) => false,
        _ => return (Err("Expected 'if' or 'unless' keyword.".to_string()), ind)
    };

    return match parse_expression(it, ind) {
        (Ok(cond), ind) => if it.next() != Some(&Token::Then) {
            return (Err("'Then' keyword expected".to_string()), ind);
        } else {
            match parse_expr_or_stmt(it, ind) {
                (Ok(then), ind) => if Some(&&Token::Else) != it.peek() {
                    if if_expr {
                        (Ok(ASTNode::If(wrap!(cond), wrap!(then))), ind)
                    } else {
                        (Ok(ASTNode::Unless(wrap!(cond), wrap!(then))), ind)
                    }
                } else {
                    it.next();
                    match parse_expr_or_stmt(it, ind) {
                        (Ok(otherwise), ind) => if if_expr {
                            (Ok(ASTNode::IfElse(wrap!(cond), wrap!(then), wrap!(otherwise))), ind)
                        } else {
                            (Ok(ASTNode::UnlessElse(wrap!(cond), wrap!(then), wrap!(otherwise))),
                             ind)
                        }
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}

fn parse_from(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    if it.next() != Some(&Token::From) { return (Err("Expected 'from' keyword".to_string()), ind); }

    return match parse_expression(it, ind) {
        (Ok(coll), ind) => if it.next() == Some(&Token::Where) {
            match parse_expression(it, ind) {
                (Ok(cond), ind) => if it.peek() == Some(&&Token::Map) {
                    match (it.next(), parse_function_anonymous(it, ind)) {
                        (_, (Ok(mapping), ind)) => (Ok(
                            ASTNode::FromMap(wrap!(coll), wrap!(cond), wrap!(mapping))), ind),
                        (_, err) => err
                    }
                } else {
                    (Ok(ASTNode::From(wrap!(coll), wrap!(cond))), ind)
                }
                err => err
            }
        } else {
            (Err("Expected 'where' keyword.".to_string()), ind)
        }
        err => err
    };
}

fn parse_when(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    if it.next() != Some(&Token::When) { return (Err("Expected 'when' keyword".to_string()), ind); }

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
