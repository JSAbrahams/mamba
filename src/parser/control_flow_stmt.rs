use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use crate::parser::parse_expression_or_do;
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

        Some(t) => panic!(format!("Expected control flow statement, but other token: {:?}", t)),
        None => panic!("Expected control flow statement, but end of file.")
    };
}

// loop ::= "loop" expression-or-do
fn parse_loop(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Loop));

    return match parse_expression_or_do(it, ind) {
        (Ok(expr_or_do), new_ind) => (Ok(ASTNode::Loop(Box::new(expr_or_do))), new_ind),
        err => err
    };
}

// while ::= "while" expression "do" expression-or-do
fn parse_while(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::While));

    return match parse_expression(it, ind) {
        (Ok(cond), new_ind) => {
            if it.next() != Some(&Token::Do) {
                return (Err("Expected 'do' after while conditional.".to_string()), new_ind);
            }

            match parse_expression_or_do(it, new_ind) {
                (Ok(expr_or_do), nnew_ind) =>
                    (Ok(ASTNode::While(Box::new(cond), Box::new(expr_or_do))),
                     nnew_ind),
                err => err
            }
        }
        err => err
    };
}

// for ::= "for" expression "in" expression "do" expression-or-do
fn parse_for(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::For));

    return match parse_expression(it, ind) {
        (Ok(expr), new_ind) => {
            if it.next() != Some(&Token::In) {
                return (Err("Expected 'in' after for expression".to_string()), new_ind);
            }

            match parse_expression(it, new_ind) {
                (Ok(col), nnew_ind) => {
                    if it.next() != Some(&Token::Do) {
                        return (Err("Expected 'do' after for collection".to_string()), new_ind);
                    }

                    match parse_expression_or_do(it, nnew_ind) {
                        (Ok(expr_or_do), nnnew_ind) =>
                            (Ok(ASTNode::For(Box::new(expr), Box::new(col),
                                             Box::new(expr_or_do))), nnnew_ind),
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}