use crate::lexer::Token;
use crate::parser::assignment::parse_assignment;
use crate::parser::ASTNode;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::maybe_expr::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// statement ::=
// | "print" maybe-expr
// | assignment
// | control-flow-stmt

pub fn parse_statement(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Print) => match (it.next(), parse_expression(it, ind)) {
            (_, (Ok(expr), ind)) => (Ok(ASTNode::Print(wrap!(expr))), ind),
            (_, err) => err
        }
        Some(Token::Let) | Some(Token::Mut) => parse_assignment(it, ind),
        Some(Token::For) | Some(Token::While) | Some(Token::Loop) => parse_cntrl_flow_stmt(it, ind),

        Some(_) | None => (Err("Expected statement.".to_string()), ind)
    };
}
