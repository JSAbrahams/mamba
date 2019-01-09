use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// reassignment     ::= maybe-expr "<-" maybe-expr
// assignment       ::= mutable-assign | immutable-assign
// mutable-assign   ::= [ "mutable" ] immutable-assignment
// immutable-assign ::= variable-def "<-" maybe-expr
// definition       ::= "let" id

pub fn parse_assignment(it: &mut Peekable<Iter<Token>>, ind: i32)
                        -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) => parse_nor_assign(it, ind),
        Some(Token::Mut) => parse_mut_assign(it, ind),

        Some(_) | None => (Err("Expected assignment.".to_string()), ind)
    };
}

fn parse_nor_assign(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match (it.next(), parse_id(it, ind), it.next()) {
        (Some(Token::Let), (Ok(id), ind), Some(Token::Assign)) => match parse_expression(it, ind) {
            (Ok(expr), ind) => (Ok(ASTNode::Assign(wrap!(id), wrap!(expr))), ind),
            err => err
        }
        (_, (Ok(_), _), Some(Token::Assign)) => (Err("Expected 'let'.".to_string()), ind),
        (Some(Token::Let), (Ok(_), _), _) => (Err("expected 'assign'.".to_string()), ind),
        (_, err, _) => err,
    }
}

fn parse_id(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.next() {
        Some(Token::Id(id)) => (Ok(ASTNode::Id(id.to_string())), ind),
        Some(_) | None => (Err("expected an id.".to_string()), ind)
    };
}

fn parse_mut_assign(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Mut));

    match parse_nor_assign(it, ind) {
        (Ok(assign), ind) => (Ok(ASTNode::Mut(wrap!(assign))), ind),
        err => err
    }
}
