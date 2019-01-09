use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::statement::parse_statement;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// expr-or-stmt ::=
// | statement
// | maybe-expr [ ( "if" | "unless" ) maybe_expr ]

macro_rules! postfix_op { ($it:expr, $ind:expr, $op:path, $pre:expr) => {{
    $it.next(); match parse_expression($it, $ind) {
        (Ok(post), nind) => (Ok($op(Box::new($pre), Box::new(post))), nind),
        err => err
    }
}}}

pub fn parse_expr_or_stmt(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                          -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos::Let) | Some(TokenPos::Mut) | Some(TokenPos::Print) | Some(TokenPos::For) |
        Some(TokenPos::While) | Some(TokenPos::Loop) => parse_statement(it, ind),
        _ => parse_expression(it, ind)
    } {
        (Ok(pre), ind) => match it.peek() {
            Some(TokenPos::If) => postfix_op!(it, ind, ASTNode::If, pre),
            Some(TokenPos::Unless) => postfix_op!(it, ind, ASTNode::Unless, pre),
            Some(_) | None => (Ok(pre), ind)
        }
        err => err
    };
}
