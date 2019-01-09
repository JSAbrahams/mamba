use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::statement::parse_statement;
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

pub fn parse_expr_or_stmt(it: &mut Peekable<Iter<Token>>, ind: i32)
                          -> (Result<ASTNode, String>, i32) {
    return match match it.peek() {
        Some(Token::Let) | Some(Token::Mut) | Some(Token::Print) | Some(Token::For) |
        Some(Token::While) | Some(Token::Loop) => parse_statement(it, ind),
        _ => parse_expression(it, ind)
    } {
        (Ok(pre), ind) => match it.peek() {
            Some(Token::If) => postfix_op!(it, ind, ASTNode::If, pre),
            Some(Token::Unless) => postfix_op!(it, ind, ASTNode::Unless, pre),
            Some(_) | None => (Ok(pre), ind)
        }
        err => err
    };
}
