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
    $it.next();
    let (post, ind) = get_or_err!($it, $ind, parse_expression, "post operator");
    Ok(($op(post, Box::new($pre)), ind))
}}}

pub fn parse_expr_or_stmt(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    let fun = match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Let }) |
        Some(TokenPos { line: _, pos: _, token: Token::Mut }) |
        Some(TokenPos { line: _, pos: _, token: Token::Print }) |
        Some(TokenPos { line: _, pos: _, token: Token::For }) |
        Some(TokenPos { line: _, pos: _, token: Token::While }) => parse_statement,
        _ => parse_expression
    };

    let (pre, ind) = get_or_err_direct!(it, ind, fun, "expression or statement");
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::If }) => pos_op!(it, ind, ASTNode::If, pre),
        Some(TokenPos { line: _, pos: _, token: Token::Unless }) =>
            pos_op!(it, ind, ASTNode::Unless, pre),
        _ => Ok((pre, ind))
    };
}
