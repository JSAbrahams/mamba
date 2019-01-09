use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-stmt::= loop | while | for | "break" | "continue"
// while            ::= "while" maybe-expr "do" expr-or-stmt
// for              ::= "for" maybe-expr "in" maybe-expr "do" expr-or-stmt

pub fn parse_cntrl_flow_stmt(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::While) => parse_while(it, ind),
        Some(Token::For) => parse_for(it, ind),
        Some(Token::Break) => next_and!(it, (Ok(ASTNode::Break), ind)),
        Some(Token::Continue) => next_and!(it, (Ok(ASTNode::Continue), ind)),
        Some(_) | None => (Err("Expected control flow statement.".to_string()), ind)
    };
}

fn parse_while(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    if it.next() != Some(&Token::While) {
        return (Err("Expected 'while' keyword".to_string()), ind);
    }

    return match parse_expression(it, ind) {
        (Ok(cond), ind) => if let Some(&Token::Do) = it.next() {
            match parse_expr_or_stmt(it, ind) {
                (Ok(expr_or_do), ind) => (Ok(ASTNode::While(wrap!(cond), wrap!(expr_or_do))), ind),
                err => err
            }
        } else { (Err("Expected 'do' after while conditional.".to_string()), ind) }
        err => err
    };
}

fn parse_for(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    if it.next() != Some(&Token::For) { return (Err("Expected 'for' keyword".to_string()), ind); }

    return match parse_expression(it, ind) {
        (Ok(expr), ind) => if let Some(&Token::In) = it.next() {
            match parse_expression(it, ind) {
                (Ok(col), ind) => if let Some(&Token::Do) = it.next() {
                    match parse_expr_or_stmt(it, ind) {
                        (Ok(expr_or_do), ind) =>
                            (Ok(ASTNode::For(wrap!(expr), wrap!(col), wrap!(expr_or_do))), ind),
                        err => err
                    }
                } else {
                    return (Err("Expected 'do' after for collection".to_string()), ind);
                }
                err => err
            }
        } else {
            (Err("Expected 'in' after for expression".to_string()), ind)
        }
        err => err
    };
}
