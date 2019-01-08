use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse as parse_expr_or_stmt;
use crate::parser::expression_or_statement::parse_maybe_expression as parse_maybe_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-stmt ::= loop | while | for | "break" | "continue"
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Loop) => parse_loop(it, ind),
        Some(Token::While) => parse_while(it, ind),
        Some(Token::For) => parse_for(it, ind),
        Some(Token::Break) => next_and!(it, (Ok(ASTNode::Break), ind)),
        Some(Token::Continue) => next_and!(it, (Ok(ASTNode::Continue), ind)),
        Some(_) | None => panic!("Expected control flow statement.")
    };
}

// loop ::= "loop" expr-or-stmt
fn parse_loop(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Loop));

    return match parse_expr_or_stmt(it, ind) {
        (Ok(expr_or_do), new_ind) => (Ok(ASTNode::Loop(wrap!(expr_or_do))), new_ind),
        err => err
    };
}

// while ::= "while" maybe-expr "do" expr-or-stmt
fn parse_while(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::While));

    return match parse_maybe_expression(it, ind) {
        (Ok(cond), new_ind) => if let Some(&Token::Do) = it.next() {
            match parse_expr_or_stmt(it, new_ind) {
                (Ok(expr_or_do), nnew_ind) => (Ok(ASTNode::While(wrap!(cond), wrap!(expr_or_do))),
                                               nnew_ind),
                err => err
            }
        } else {
            (Err("Expected 'do' after while conditional.".to_string()), new_ind)
        }
        err => err
    };
}

// for ::= "for" maybe-expr "in" maybe-expr "do" expr-or-stmt
fn parse_for(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::For));

    return match parse_maybe_expression(it, ind) {
        (Ok(expr), new_ind) => if let Some(&Token::In) = it.next() {
            match parse_maybe_expression(it, new_ind) {
                (Ok(col), nnew_ind) => if let Some(&Token::Do) = it.next() {
                    match parse_expr_or_stmt(it, nnew_ind) {
                        (Ok(expr_or_do), nnnew_ind) =>
                            (Ok(ASTNode::For(wrap!(expr), wrap!(col), wrap!(expr_or_do))),
                             nnnew_ind),
                        err => err
                    }
                } else {
                    return (Err("Expected 'do' after for collection".to_string()), new_ind);
                }
                err => err
            }
        } else {
            (Err("Expected 'in' after for expression".to_string()), new_ind)
        }
        err => err
    };
}
