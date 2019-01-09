use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::do_block::parse_do_block;
use crate::parser::function::parse_function_call;
use crate::parser::function::parse_function_call_direct;
use crate::parser::operation::parse_operation;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

macro_rules! postfix_op { ($it:expr, $ind:expr, $op:path, $pre:expr) => {{
    $it.next(); match parse_expression($it, $ind) {
        (Ok(post), ind) => (Ok($op(Box::new($pre), Box::new(post))), ind),
        err => err
    }
}}}

// maybe-expr ::=
// | "return" [ maybe-expr ]
// | operation
// | tuple
// | control-flow-expr
// | reassignment
// | function-call
// | function-call-dir
// | newline do-block

pub fn parse_expression(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match match it.peek() {
        Some(Token::If) | Some(Token::Unless) | Some(Token::When) => parse_cntrl_flow_expr(it, ind),
        Some(Token::NL) => next_and!(it, parse_do_block(it, ind + 1)),
        Some(Token::LPar) => parse_tuple(it, ind),
        Some(Token::Ret) => parse_return(it, ind),
        Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) | Some(Token::Id(_)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) | Some(Token::Not) | Some(Token::Add) |
        Some(Token::Sub) => parse_operation(it, ind),

        Some(_) | None => (Err("Expected expression.".to_string()), ind)
    } {
        (Ok(pre), ind) => match it.peek() {
            Some(Token::Assign) => postfix_op!(it, ind, ASTNode::Assign, pre),
            Some(Token::LPar) => parse_function_call_direct(pre, it, ind),
            Some(Token::Point) => parse_function_call(pre, it, ind),
            Some(_) | None => (Ok(pre), ind)
        }
        err => err
    };
}

// tuple ::= "(" [ ( maybe-expr { "," maybe-expr } ] ")"
pub fn parse_tuple(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::LPar));

    let mut elements = Vec::new();
    if it.peek() != Some(&&Token::RPar) {
        match parse_expression(it, ind) {
            (Ok(maybe_expr), _) => elements.push(maybe_expr),
            (Err(err), ind) => return (Err(err), ind)
        }
    }

    while let Some(t) = it.next() {
        if *t == Token::RPar { break; }
        match *t {
            Token::Comma => match parse_expression(it, ind) {
                (Ok(fun_type), _) => elements.push(fun_type),
                (Err(err), ind) => return (Err(err), ind)
            }
            _ => return (Err("Expected expression.".to_string()), ind)
        };
    }

    return (Ok(ASTNode::FunTuple(elements)), ind);
}

// "return" maybe-expression
fn parse_return(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Ret));
    if it.peek() == Some(&&Token::NL) { return (Ok(ASTNode::ReturnEmpty), ind); }

    return match parse_expression(it, ind) {
        (Ok(expr), ind) => (Ok(ASTNode::Return(wrap!(expr))), ind),
        err => err
    };
}
