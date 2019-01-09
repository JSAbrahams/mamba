use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::assignment::parse_assignment;
use crate::parser::ASTNode;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseError;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// statement ::=
// | "print" maybe-expr
// | assignment
// | control-flow-stmt

pub fn parse_statement(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Print }) => match (it.next(), parse_expression(it, ind)) {
            (_, (Ok(expr), ind)) => (Ok(ASTNode::Print(wrap!(expr))), ind),
            (_, err) => err
        }

        Some(TokenPos { line, pos, token: Token::Let }) |
        Some(TokenPos { line, pos, token: Token::Mut }) => parse_assignment(it, ind),

        Some(TokenPos { line, pos, token: Token::For }) |
        Some(TokenPos { line, pos, token: Token::While }) => parse_cntrl_flow_stmt(it, ind),

        Some(tp) => (Err(ParseError::TokenError(**tp, Token::Print)), ind),
        None => (Err(ParseError::EOFError(Token::Print)), ind)
    };
}
