use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse_maybe_expression as parse_maybe_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// assignment ::= normal-assignment | mutable-assignment
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) => parse_nor_assign(it, ind),
        Some(Token::Mut) => parse_mut_assign(it, ind),

        Some(_) => panic!("token not recognized"),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}

// normal-assignment ::= "let" id "<-" maybe-expr
fn parse_nor_assign(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match (it.next(), parse_id(it, ind), it.next()) {
        (Some(Token::Let), (Ok(id), new_ind), Some(Token::Assign)) => {
            match parse_maybe_expression(it, new_ind) {
                (Ok(expr), nnew_ind) => (Ok(ASTNode::Assign(wrap!(id), wrap!(expr))), nnew_ind),
                err => err
            }
        }
        (_, (Ok(_), _), Some(Token::Assign)) => (Err("Expected 'let'.".to_string()), ind),
        (Some(Token::Let), (Ok(_), _), _) => (Err("expected 'assign'.".to_string()), ind),
        (_, err, _) => err,
    }
}

// id ::= { character }
fn parse_id(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.next() {
        Some(Token::Id(id)) => (Ok(ASTNode::Id(id.to_string())), ind),
        Some(_) | None => (Err("expected an id.".to_string()), ind)
    };
}

// mutable-assignment ::= "mutable" assignment
fn parse_mut_assign(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Mut));

    match parse_nor_assign(it, ind) {
        (Ok(assign), new_indent) => (Ok(ASTNode::Mut(wrap!(assign))), new_indent),
        err => err
    }
}
