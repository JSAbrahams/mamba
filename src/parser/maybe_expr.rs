use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::assignment::parse_reassignment;
use crate::parser::ASTNode;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::do_block::parse_do_block;
use crate::parser::function::parse_function_call;
use crate::parser::function::parse_function_call_direct;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::ParseError;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// maybe-expr ::=
// | "return" [ maybe-expr ]
// | operation
// | tuple
// | control-flow-expr
// | reassignment
// | function-call
// | function-call-dir
// | newline do-block

pub fn parse_expression(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match match it.peek() {
        Some(TokenPos { line, pos, token: Token::If }) |
        Some(TokenPos { line, pos, token: Token::Unless }) |
        Some(TokenPos { line, pos, token: Token::When }) => parse_cntrl_flow_expr(it, ind),
        Some(TokenPos { line, pos, token: Token::NL }) => next_and!(it, parse_do_block(it, ind + 1)),
        Some(TokenPos { line, pos, token: Token::LPar }) => parse_tuple(it, ind),
        Some(TokenPos { line, pos, token: Token::Ret }) => parse_return(it, ind),
        Some(TokenPos { line, pos, token: Token::Real(_) }) |
        Some(TokenPos { line, pos, token: Token::Int(_) }) |
        Some(TokenPos { line, pos, token: Token::ENum(_, _) }) |
        Some(TokenPos { line, pos, token: Token::Id(_) }) |
        Some(TokenPos { line, pos, token: Token::Str(_) }) |
        Some(TokenPos { line, pos, token: Token::Bool(_) }) |
        Some(TokenPos { line, pos, token: Token::Not }) |
        Some(TokenPos { line, pos, token: Token::Add }) |
        Some(TokenPos { line, pos, token: Token::Sub }) => parse_operation(it, ind),

        Some(tp) => (Err(ParseError::TokenError(**tp, Token::If)), ind),
        None => (Err(ParseError::EOFError(Token::If)), ind)
    } {
        (Ok(pre), ind) => match it.peek() {
            Some(TokenPos { line, pos, token: Token::Assign }) => parse_reassignment(pre, it, ind),
            Some(TokenPos { line, pos, token: Token::LPar }) =>
                parse_function_call_direct(pre, it, ind),
            Some(TokenPos { line, pos, token: Token::Point }) => parse_function_call(pre, it, ind),
            Some(_) | None => (Ok(pre), ind)
        }
        err => err
    };
}

// tuple ::= "(" [ ( maybe-expr { "," maybe-expr } ] ")"
pub fn parse_tuple(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::LPar =>
            return (Err(ParseError::TokenError(*tp, Token::LPar)), ind),
        None => return (Err(ParseError::EOFError(Token::LPar)), ind)
    }

    let mut elements = Vec::new();
    if it.peek() != Some(&&TokenPos::RPar) {
        match parse_expression(it, ind) {
            (Ok(maybe_expr), _) => elements.push(maybe_expr),
            (Err(err), ind) => return (Err(err), ind)
        }
    }

    while let Some(t) = it.next() {
        match *t {
            TokenPos { line, pos, token: Token::RPar } => break,
            TokenPos { line, pos, token: Token::Comma } => match parse_expression(it, ind) {
                (Ok(fun_type), _) => elements.push(fun_type),
                (Err(err), ind) => return (Err(err), ind)
            }
            tp => (Err(ParseError::TokenError(**tp, Token::Comma)), ind),
        };
    }

    return (Ok(ASTNode::FunTuple(elements)), ind);
}

// "return" maybe-expression
fn parse_return(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::Ret =>
            return (Err(ParseError::TokenError(*tp, Token::Ret)), ind),
        None => return (Err(ParseError::EOFError(Token::Ret)), ind)
    }

    if it.peek() == Some(&&TokenPos::NL) { return (Ok(ASTNode::ReturnEmpty), ind); }
    return match parse_expression(it, ind) {
        (Ok(expr), ind) => (Ok(ASTNode::Return(wrap!(expr))), ind),
        err => err
    };
}
