use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// identifier ::= assignment | mutable-assignment
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) => parse_assignment(it, ind),
        Some(Token::Mut) => parse_mut_assign(it, ind),

        Some(_) => panic!("token not recognized"),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}

// assignment ::= "let" id "<-" expression
fn parse_assignment(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) => {
            it.next();
            match parse_id(it, ind) {
                (Ok(id), new_ind) => {
                    if it.next() != Some(&Token::Assign) {
                        return (Err("Expected Assign token".to_string()), ind);
                    }
                    match parse_expression(it, new_ind) {
                        (Ok(expr), nnew_ind) =>
                            (Ok(ASTNode::Assign(Box::new(id), Box::new(expr))),
                             nnew_ind),
                        err => err
                    }
                }
                err => err
            }
        }

        Some(_) => panic!("token not recognized"),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}

// id ::= { character }
fn parse_id(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.next() {
        Some(Token::Id(id)) => (Ok(ASTNode::Id(id.to_string())), ind),

        Some(_) => panic!("Expected id, but other token."),
        None => panic!("Expected id, but end of file.")
    };
}

// mutable-assignment ::= "mutable" assignment
fn parse_mut_assign(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Mut));

    match parse_assignment(it, ind) {
        (Ok(assign), new_indent) => (Ok(ASTNode::Mut(Box::new(assign))), new_indent),
        err => err
    }
}
