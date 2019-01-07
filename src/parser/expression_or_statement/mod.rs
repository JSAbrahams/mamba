use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression::parse as parse_expression;
use crate::parser::program::parse_function_call_direct;
use crate::parser::program::parse_do;
use crate::parser::program::parse_function_call;
use crate::parser::statement::parse as parse_statement;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

mod control_flow;

macro_rules! postfix_op { ($it:expr, $ind:expr, $op:path, $pre:expr) => {{
    $it.next();
    match parse_maybe_expression($it, $ind) {
        (Ok(post), nnew_ind) => (Ok($op(Box::new($pre), Box::new(post))), nnew_ind),
        err => err
    }
}}}

// maybe-expr ::= expression | "(" ( maybe-expr [ "," maybe-expr] ")" | control-flow-expr
//             | function-call | maybe-expr "<-" maybe-expr | function-call | newline do-block
pub fn parse_maybe_expression(it: &mut Peekable<Iter<Token>>, ind: i32)
                              -> (Result<ASTNode, String>, i32) {
    return match match it.peek() {
        Some(Token::If) | Some(Token::Unless) | Some(Token::When) => control_flow::parse(it, ind),
        Some(Token::NL) => next_and!(it, parse_do(it, ind + 1)),
        Some(Token::LPar) => {
            it.next();
            match parse(it, ind) {
                (Ok(expr_or_stmt), new_ind) => match it.next() {
                    Some(&Token::RPar) => (Ok(expr_or_stmt), new_ind),
                    Some(&Token::Comma) => {
                        let mut elements = Vec::new();
                        elements.push(expr_or_stmt);

                        while Some(&&Token::Comma) != it.peek()
                            && Some(&&Token::RPar) != it.peek() {
                            match parse(it, ind) {
                                (Ok(element), _) => elements.push(element),
                                err => return err
                            }
                        }

                        if it.next() != Some(&Token::RPar) {
                            (Err("Expected closing bracket after tuple.".to_string()), new_ind)
                        } else {
                            (Ok(ASTNode::Tuple(elements)), new_ind)
                        }
                    }
                    _ => (Err("Expected either closing bracket after expression or statement, or \
                    comma between tuple elements.".to_string()), new_ind)
                }
                err => err
            }
        }
        _ => parse_expression(it, ind)
    } {
        (Ok(pre), new_ind) => match it.peek() {
            Some(Token::Assign) => postfix_op!(it, new_ind, ASTNode::Assign, pre),
            Some(Token::LPar) => parse_function_call_direct(pre, it, ind),
            Some(Token::Point) => parse_function_call(pre, it, ind),
            Some(_) | None => (Ok(pre), new_ind)
        }
        err => err
    };
}

// tuple ::= "(" [ ( maybe-expr { "," maybe-expr } ] ")"
pub fn parse_tuple(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::LPar));

    let mut elements = Vec::new();
    if it.peek() != Some(&&Token::RPar) {
        match parse_maybe_expression(it, ind) {
            (Ok(maybe_expr), _) => elements.push(maybe_expr),
            (Err(err), new_ind) => return (Err(err), new_ind)
        }
    }

    loop {
        match it.next() {
            Some(Token::Comma) => match parse_maybe_expression(it, ind) {
                (Ok(fun_type), _) => elements.push(fun_type),
                (Err(err), new_ind) => return (Err(err), new_ind)
            }
            Some(Token::RPar) => break,

            Some(t) => return (Err(format!("Expected expression, but got {:?}.", t)), ind),
            None => return (Err("Expected expression, but end of file.".to_string()), ind)
        };
    }

    return (Ok(ASTNode::StaticTuple(elements)), ind);
}

// expr-or-stmt ::= statement | maybe-expr ( [ "<-" maybe_expr ] | ( "if" | "unless" ) maybe_expr )
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match match it.peek() {
        Some(Token::Let) | Some(Token::Mut) | Some(Token::Print) | Some(Token::DoNothing) |
        Some(Token::For) | Some(Token::While) | Some(Token::Loop) => parse_statement(it, ind),
        _ => parse_maybe_expression(it, ind)
    } {
        (Ok(pre), new_ind) => match it.peek() {
            Some(Token::If) => postfix_op!(it, new_ind, ASTNode::If, pre),
            Some(Token::Unless) => postfix_op!(it, new_ind, ASTNode::Unless, pre),
            Some(_) | None => (Ok(pre), new_ind)
        }
        err => err
    };
}