use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse_maybe_expression as parse_maybe_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

mod assignment;
mod control_flow;

// statement ::= "print" maybe-expr | assignment | "donothing" | control-flow-stmt
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) | Some(Token::Mut) => assignment::parse(it, ind),
        Some(Token::Print) => {
            it.next();
            match parse_maybe_expression(it, ind) {
                (Ok(expr), new_indent) => (Ok(ASTNode::Print(wrap!(expr))), new_indent),
                err => err
            }
        },
        Some(Token::DoNothing) => (Ok(ASTNode::DoNothing), ind),
        Some(Token::For) | Some(Token::While) | Some(Token::Loop) => control_flow::parse(it, ind),

        Some(t) => (Err(format!("Unexpected token while parsing statement: {:?}", t).to_string()),
                    ind),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}
