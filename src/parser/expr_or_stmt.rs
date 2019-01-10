use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::statement::parse_statement;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// expr-or-stmt ::=
// | statement
// | maybe-expr [ ( "if" | "unless" ) maybe_expr ]

macro_rules! pos_op { ($it:expr, $ind:expr, $op:path, $pre:expr) => {{
    $it.next(); match parse_expression($it, $ind) {
        (Ok(post), nind) => (Ok($op(Box::new($pre), Box::new(post))), nind),
        err => err
    }
}}}

pub fn parse_expr_or_stmt(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                          -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos { line, pos, token: Token::Let }) |
        Some(TokenPos { line, pos, token: Token::Mut }) |
        Some(TokenPos { line, pos, token: Token::Print }) |
        Some(TokenPos { line, pos, token: Token::For }) |
        Some(TokenPos { line, pos, token: Token::While }) |
        Some(TokenPos { line, pos, token: Token::Loop }) => parse_statement(it, ind),
        _ => parse_expression(it, ind)
    } {
        (Ok(pre), ind) => match it.peek() {
            Some(TokenPos { line, pos, token: Token::If }) => pos_op!(it, ind, ASTNode::If, pre),
            Some(TokenPos { line, pos, token: Token::Unless }) =>
                pos_op!(it, ind, ASTNode::Unless, pre),

            Some(actual) => (Err(TokenErr { expected: Token::Let, actual }), ind),
            None => (Err(EOFErr { expected: Token::If }), ind)
        }
        err => err
    };
}
