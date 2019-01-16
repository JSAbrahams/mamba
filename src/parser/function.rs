use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_tuple;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::env;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_function_call(caller: ASTNode, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                           -> ParseResult<ASTNode> {
    print_parse!(it, ind, "function call");
    check_next_is!(it, Token::Point);

    match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::LPar }) => {
                let (tuple, ind) = get_or_err!(it, ind, parse_tuple, "function call");
                Ok((ASTNode::FunCall(Box::new(caller), Box::new(ASTNode::Id(id.to_string())), tuple), ind))
            }
            Some(&next) => Err(TokenErr { expected: Token::LPar, actual: next.clone() }),
            None => Err(EOFErr { expected: Token::LPar })
        }
        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}

pub fn parse_function_call_direct(name: ASTNode, it: &mut Peekable<Iter<TokenPos>>,
                                  ind: i32) -> ParseResult<ASTNode> {
    print_parse!(it, ind, "function call direct");

    match (name, it.peek()) {
        (ASTNode::Id(ref id), Some(TokenPos { line: _, pos: _, token: Token::LPar })) => {
            let (tuple, ind) = get_or_err!(it, ind, parse_tuple, "direction function call");
            Ok((ASTNode::FunCallDirect(Box::new(ASTNode::Id(id.to_string())), tuple), ind))
        }
        (_, Some(&next)) =>
            Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        (_, _) => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}

pub fn parse_function_definition_body(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                      -> ParseResult<ASTNode> {
    print_parse!(it, ind, "function definition");
    check_next_is!(it, Token::Fun);

    return match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) => {
            let (args, ind) = get_or_err_direct!(it, ind, parse_args,
                                                 "function definition with body");
            match it.next() {
                Some(TokenPos { line: _, pos: _, token: Token::To }) => {
                    let (body, ind) = get_or_err!(it, ind, parse_expr_or_stmt,
                                                  "function definition with body");
                    Ok((ASTNode::FunDefNoRetType(
                        Box::new(ASTNode::Id(id.to_string())), args, body), ind))
                }
                Some(TokenPos { line: _, pos: _, token: Token::DoublePoint }) => {
                    let (ret_type, ind) = get_or_err!(it, ind, parse_function_type,
                                                      "function definition with body");
                    match it.next() {
                        Some(TokenPos { line: _, pos: _, token: Token::To }) => {
                            let (body, ind) = get_or_err!(it, ind, parse_expr_or_stmt,
                                                          "function definition with body");
                            Ok((ASTNode::FunDef(
                                Box::new(ASTNode::Id(id.to_string())), args, ret_type, body), ind))
                        }
                        Some(next) => Err(TokenErr { expected: Token::To, actual: next.clone() }),
                        None => Err(EOFErr { expected: Token::To })
                    }
                }
                Some(next) => Err(TokenErr { expected: Token::DoublePoint, actual: next.clone() }),
                None => Err(EOFErr { expected: Token::DoublePoint })
            }
        }
        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    };
}

fn parse_args(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<Vec<ASTNode>> {
    print_parse!(it, ind, "function arguments");
    check_next_is!(it, Token::LPar);

    let mut args = Vec::new();
    loop {
        match it.next() {
            Some(TokenPos { line: _, pos: _, token: Token::RPar }) => break,
            Some(TokenPos { line: _, pos: _, token: Token::Comma }) => {
                let (fun_type, _) = get_or_err_direct!(it, ind, parse_function_arg,
                                                       "function type");
                args.push(fun_type);
            }
            Some(next) => return Err(TokenErr { expected: Token::RPar, actual: next.clone() }),
            None => return Err(EOFErr { expected: Token::RPar })
        };
    }

    return Ok((args, ind));
}

fn parse_function_arg(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    print_parse!(it, ind, "function argument");
    let (fun_arg, ind) = get_or_err!(it, ind, parse_function_type, "function argument");

    match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::DoublePoint }) => {
            let (arg_ty, ind) = get_or_err!(it, ind, parse_function_type, "function argument type");
            Ok((ASTNode::FunArg(fun_arg, arg_ty), ind))
        }
        Some(next) => Err(TokenErr { expected: Token::DoublePoint, actual: next.clone() }),
        None => Err(EOFErr { expected: Token::DoublePoint })
    }
}

fn parse_function_type(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    print_parse!(it, ind, "function type");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) =>
            next_and!(it, Ok((ASTNode::Id(id.to_string()), ind))),
        Some(TokenPos { line: _, pos: _, token: Token::LPar }) => {
            let (tup, ind) = get_or_err!(it, ind, parse_function_tuple, "function tuple");
            check_next_is!(it, Token::To);
            let (fun_ty, ind) = get_or_err!(it, ind, parse_function_type, "function type");
            Ok((ASTNode::FunType(tup, fun_ty), ind))
        }
        Some(&next) => Err(TokenErr { expected: Token::LPar, actual: next.clone() }),
        None => Err(EOFErr { expected: Token::LPar })
    };
}

fn parse_function_tuple(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    print_parse!(it, ind, "function tuple");
    check_next_is!(it, Token::LPar);

    let mut fun_types: Vec<ASTNode> = Vec::new();
    match it.next() {
        Some(next) if next.token != Token::RPar => {
            let (fun_type, _) = get_or_err_direct!(it, ind, parse_function_type, "function tuple");
            fun_types.push(fun_type)
        }
        Some(next) => return Err(TokenErr { expected: Token::RPar, actual: next.clone() }),
        None => return Err(EOFErr { expected: Token::RPar })
    }

    loop {
        match it.next() {
            Some(TokenPos { line: _, pos: _, token: Token::RPar }) => break,
            Some(TokenPos { line: _, pos: _, token: Token::Comma }) => {
                let (fun_type, _) = get_or_err_direct!(it, ind, parse_function_type,
                                                       "tuple element");
                fun_types.push(fun_type);
            }
            Some(next) => return Err(TokenErr { expected: Token::LPar, actual: next.clone() }),
            None => return Err(EOFErr { expected: Token::LPar })
        };
    }

    return Ok((ASTNode::FunTuple(fun_types), ind));
}

pub fn parse_function_anonymous(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                -> ParseResult<ASTNode> {
    let (tuple, ind) = get_or_err!(it, ind, parse_function_tuple, "anonymous function");
    check_next_is!(it, Token::To);
    let (body, ind) = get_or_err!(it, ind, parse_expr_or_stmt, "anonymous function body");
    return Ok((ASTNode::FunAnon(tuple, body), ind));
}