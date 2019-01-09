use crate::lexer::TokenPos;
use crate::parser::assignment::parse_reassignment;
use crate::parser::ASTNode;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::do_block::parse_do_block;
use crate::parser::function::parse_function_call;
use crate::parser::function::parse_function_call_direct;
use crate::parser::operation::parse_operation;
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
        Some(TokenPos::If) | Some(TokenPos::Unless) | Some(TokenPos::When) => parse_cntrl_flow_expr(it, ind),
        Some(TokenPos::NL) => next_and!(it, parse_do_block(it, ind + 1)),
        Some(TokenPos::LPar) => parse_tuple(it, ind),
        Some(TokenPos::Ret) => parse_return(it, ind),
        Some(TokenPos::Real(_)) | Some(TokenPos::Int(_)) | Some(TokenPos::ENum(_, _)) | Some(TokenPos::Id(_)) |
        Some(TokenPos::Str(_)) | Some(TokenPos::Bool(_)) | Some(TokenPos::Not) | Some(TokenPos::Add) |
        Some(TokenPos::Sub) => parse_operation(it, ind),

        Some(_) | None => (Err("Expected expression.".to_string()), ind)
    } {
        (Ok(pre), ind) => match it.peek() {
            Some(TokenPos::Assign) => parse_reassignment(pre, it, ind),
            Some(TokenPos::LPar) => parse_function_call_direct(pre, it, ind),
            Some(TokenPos::Point) => parse_function_call(pre, it, ind),
            Some(_) | None => (Ok(pre), ind)
        }
        err => err
    };
}

// tuple ::= "(" [ ( maybe-expr { "," maybe-expr } ] ")"
pub fn parse_tuple(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    if it.next() != Some(&TokenPos::LPar) {
        return (Err("Expected opening parenthesis.".to_string()), ind);
    }

    let mut elements = Vec::new();
    if it.peek() != Some(&&TokenPos::RPar) {
        match parse_expression(it, ind) {
            (Ok(maybe_expr), _) => elements.push(maybe_expr),
            (Err(err), ind) => return (Err(err), ind)
        }
    }

    while let Some(t) = it.next() {
        if *t == TokenPos::RPar { break; }
        match *t {
            TokenPos::Comma => match parse_expression(it, ind) {
                (Ok(fun_type), _) => elements.push(fun_type),
                (Err(err), ind) => return (Err(err), ind)
            }
            _ => return (Err("Expected expression.".to_string()), ind)
        };
    }

    return (Ok(ASTNode::FunTuple(elements)), ind);
}

// "return" maybe-expression
fn parse_return(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    if it.next() != Some(&TokenPos::Ret) {
        return (Err("Expected 'return' keyword".to_string()), ind);
    }

    if it.peek() == Some(&&TokenPos::NL) { return (Ok(ASTNode::ReturnEmpty), ind); }
    return match parse_expression(it, ind) {
        (Ok(expr), ind) => (Ok(ASTNode::Return(wrap!(expr))), ind),
        err => err
    };
}
