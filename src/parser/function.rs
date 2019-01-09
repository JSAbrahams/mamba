use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_tuple;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// function-call    ::= maybe-expr "." id tuple
// function-call-dir::= id tuple
// function-def     ::= "fun" id "(" function-args ")" [ ":" function-type ]
// function-def-bod ::= function-def "->" expr-or-stmt
// function-args    ::= function-type ":" function-type [ "," function-args ]
// function-type    ::= id | static-tuple | function-tuple "->" function-type
// function-tuple   ::= "(" [ function-type { "," function-type } ] ")"

pub fn parse_function_call(caller: ASTNode, it: &mut Peekable<Iter<Token>>, ind: i32)
                           -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Point));

    match (it.next(), it.peek()) {
        (Some(Token::Id(id)), Some(Token::LPar)) =>
            match parse_tuple(it, ind) {
                (Ok(tuple), ind) => (Ok(ASTNode::FunCall(
                    wrap!(caller), wrap!(ASTNode::Id(id.to_string())), wrap!(tuple))), ind),
                err => err
            }
        (_, Some(Token::LPar)) => (Err("Expected identifier.".to_string()), ind),
        (_, _) => (Err("Expected opening bracket.".to_string()), ind),
    }
}

pub fn parse_function_call_direct(function: ASTNode, it: &mut Peekable<Iter<Token>>, ind: i32)
                                  -> (Result<ASTNode, String>, i32) {
    match (function, it.peek()) {
        (ASTNode::Id(ref id), Some(Token::LPar)) => match parse_tuple(it, ind) {
            (Ok(tuple), ind) =>
                (Ok(ASTNode::FunCallDirect(wrap!(ASTNode::Id(id.to_string())), wrap!(tuple))), ind),
            err => err
        }
        (_, Some(Token::LPar)) => (Err("Expected identifier.".to_string()), ind),
        (_, _) => (Err("Expected opening bracket.".to_string()), ind),
    }
}

pub fn parse_function_definition_body(it: &mut Peekable<Iter<Token>>, ind: i32)
                                      -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Fun));

    return if let Some(Token::Id(id)) = it.next() {
        match parse_args(it, ind) {
            (Ok(args), ind) => match it.next() {
                Some(Token::To) => match parse_expr_or_stmt(it, ind) {
                    (Ok(body), ind) => (Ok(ASTNode::FunDefNoRetType(
                        wrap!(ASTNode::Id(id.to_string())), args, wrap!(body))), ind),
                    err => err
                }
                Some(Token::DoublePoint) => match parse_function_type(it, ind) {
                    (Ok(ret_type), ind) => match it.next() {
                        Some(Token::To) => match parse_expr_or_stmt(it, ind) {
                            (Ok(body), ind) => (Ok(ASTNode::FunDef(
                                wrap!(ASTNode::Id(id.to_string())),
                                args,
                                wrap!(ret_type),
                                wrap!(body))), ind),
                            err => err
                        }
                        Some(_) | None => (Err("Expected function body.".to_string()), ind)
                    },
                    err => err
                }
                Some(_) | None => (Err("Expected either 'is' or function return type.".to_string()),
                                   ind)
            }
            (Err(err), ind) => (Err(err), ind)
        }
    } else {
        (Err("Expected function name".to_string()), ind)
    };
}

fn parse_args(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<Vec<ASTNode>, String>, i32) {
    return if let Some(Token::LPar) = it.next() {
        let mut args = Vec::new();
        if it.peek() != Some(&&Token::RPar) {
            match parse_function_arg(it, ind) {
                (Ok(arg), _) => args.push(arg),
                (Err(err), ind) => return (Err(err), ind)
            }
        }

        loop {
            match it.next() {
                Some(Token::Comma) => match parse_function_arg(it, ind) {
                    (Ok(fun_type), _) => args.push(fun_type),
                    (Err(err), ind) => return (Err(err), ind)
                }
                Some(Token::RPar) => break,

                Some(_) | None => return (Err(
                    "Expected closing bracket after function arguments".to_string()), ind)
            };
        }

        (Ok(args), ind)
    } else {
        (Err("Expected opening bracket for arguments".to_string()), ind)
    };
}

fn parse_function_arg(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match parse_function_type(it, ind) {
        (Ok(arg), ind) => match it.next() {
            Some(Token::DoublePoint) => match parse_function_type(it, ind) {
                (Ok(ty), ind) => (Ok(ASTNode::FunArg(wrap!(arg), wrap!(ty))), ind),
                err => err
            }
            Some(_) | None => (Err("Expected double point after argument id.".to_string()), ind)
        },
        err => err
    }
}

fn parse_function_type(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(id)) => next_and!(it, (Ok(ASTNode::Id(id.to_string())), ind)),
        Some(Token::LPar) => match parse_static_tuple(it, ind) {
            (Ok(tup), ind) => if let Some(Token::To) = it.peek() {
                it.next();
                match parse_function_type(it, ind) {
                    (Ok(fun_ty), ind) => (Ok(ASTNode::FunType(wrap!(tup), wrap!(fun_ty))), ind),
                    err => err
                }
            } else { (Ok(tup), ind) }
            err => err
        }
        Some(_) | None => (Err("Expected function type.".to_string()), ind)
    };
}

fn parse_static_tuple(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::LPar));

    let mut fun_types = Vec::new();
    if it.peek() != Some(&&Token::RPar) {
        match parse_function_type(it, ind) {
            (Ok(fun_type), _) => fun_types.push(fun_type),
            err => return err
        }
    }

    loop {
        match it.next() {
            Some(Token::RPar) => break,

            Some(Token::Comma) => match parse_function_type(it, ind) {
                (Ok(fun_type), _) => fun_types.push(fun_type),
                err => return err
            }
            Some(_) | None => return (Err("Expected function type.".to_string()), ind)
        };
    }

    return (Ok(ASTNode::FunTuple(fun_types)), ind);
}
