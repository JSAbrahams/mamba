use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_tuple;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
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
// function-anon    ::= ( id | function-tuple ) "->' maybe-expr

pub fn parse_function_call(caller: ASTNode, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                           -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::Point);
    match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::LPar }) => match parse_tuple(it, ind) {
                (Ok(tuple), ind) => (Ok(ASTNode::FunCall(
                    wrap!(caller), wrap!(ASTNode::Id(id.to_string())), wrap!(tuple))), ind),
                err => err
            }
            Some(&next) => (Err(TokenErr { expected: Token::LPar, actual: next.clone() }), ind),
            None => (Err(EOFErr { expected: Token::LPar }), ind)
        }
        Some(next) =>
            (Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }), ind),
        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    }
}

pub fn parse_function_call_direct(function: ASTNode, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                  -> (ParseResult<ASTNode>, i32) {
    match (function, it.peek()) {
        (ASTNode::Id(ref id), Some(TokenPos { line: _, pos: _, token: Token::LPar })) =>
            match parse_tuple(it, ind) {
                (Ok(tuple), ind) =>
                    (Ok(ASTNode::FunCallDirect(wrap!(ASTNode::Id(id.to_string())), wrap!(tuple))), ind),
                err => err
            }
        (_, Some(&next)) =>
            (Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }), ind),
        (_, _) => (Err(EOFErr { expected: Token::Id(String::new()) }), ind),
    }
}

pub fn parse_function_definition_body(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                      -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::Fun);
    return match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) => match parse_args(it, ind) {
            (Ok(args), ind) => match it.next() {
                Some(TokenPos { line: _, pos: _, token: Token::To }) =>
                    match parse_expr_or_stmt(it, ind) {
                        (Ok(body), ind) => (Ok(ASTNode::FunDefNoRetType(
                            wrap!(ASTNode::Id(id.to_string())), args, wrap!(body))), ind),
                        err => err
                    }
                Some(TokenPos { line: _, pos: _, token: Token::DoublePoint }) =>
                    match parse_function_type(it, ind) {
                        (Ok(ret_type), ind) => match it.next() {
                            Some(TokenPos { line: _, pos: _, token: Token::To }) =>
                                match parse_expr_or_stmt(it, ind) {
                                    (Ok(body), ind) => (Ok(ASTNode::FunDef(
                                        wrap!(ASTNode::Id(id.to_string())),
                                        args,
                                        wrap!(ret_type),
                                        wrap!(body))), ind),
                                    err => err
                                }
                            Some(next) =>
                                (Err(TokenErr { expected: Token::To, actual: next.clone() }), ind),
                            None => (Err(EOFErr { expected: Token::To }), ind)
                        },
                        err => err
                    }
                Some(next) =>
                    (Err(TokenErr { expected: Token::DoublePoint, actual: next.clone() }), ind),
                None => (Err(EOFErr { expected: Token::DoublePoint }), ind)
            }
            (Err(err), ind) => (Err(err), ind)
        }

        Some(next) =>
            (Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }), ind),
        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    };
}

fn parse_args(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<Vec<ASTNode>>, i32) {
    check_next_is!(it, ind, Token::LPar);

    let mut args = Vec::new();
    if let Some(&&TokenPos { line: _, pos: _, token: Token::RPar }) = it.peek() {
        match parse_function_arg(it, ind) {
            (Ok(arg), _) => args.push(arg),
            (Err(err), ind) => return (Err(err), ind)
        }
    }

    loop {
        match it.next() {
            Some(TokenPos { line: _, pos: _, token: Token::RPar }) => break,
            Some(TokenPos { line: _, pos: _, token: Token::Comma }) =>
                match parse_function_arg(it, ind) {
                    (Ok(fun_type), _) => args.push(fun_type),
                    (Err(err), ind) => return (Err(err), ind)
                }
            Some(next) =>
                return (Err(TokenErr { expected: Token::RPar, actual: next.clone() }), ind),
            None => return (Err(EOFErr { expected: Token::RPar }), ind)
        };
    }

    (Ok(args), ind)
}

fn parse_function_arg(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match parse_function_type(it, ind) {
        (Ok(arg), ind) => match it.next() {
            Some(TokenPos { line: _, pos: _, token: Token::DoublePoint }) =>
                match parse_function_type(it, ind) {
                    (Ok(ty), ind) => (Ok(ASTNode::FunArg(wrap!(arg), wrap!(ty))), ind),
                    err => err
                }

            Some(next) =>
                (Err(TokenErr { expected: Token::DoublePoint, actual: next.clone() }), ind),
            None => (Err(EOFErr { expected: Token::DoublePoint }), ind)
        },
        err => err
    }
}

fn parse_function_type(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) =>
            next_and!(it, (Ok(ASTNode::Id(id.to_string())), ind)),
        Some(TokenPos { line: _, pos: _, token: Token::LPar }) =>
            match parse_function_tuple(it, ind) {
                (Ok(tup), ind) => {
                    check_next_is!(it, ind, Token::To);
                    match parse_function_type(it, ind) {
                        (Ok(fun_ty), ind) => (Ok(ASTNode::FunType(wrap!(tup), wrap!(fun_ty))),
                                              ind),
                        err => err
                    }
                }
                err => err
            }
        Some(&next) => (Err(TokenErr { expected: Token::LPar, actual: next.clone() }), ind),
        None => (Err(EOFErr { expected: Token::LPar }), ind)
    };
}

fn parse_function_tuple(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                        -> (ParseResult<ASTNode>, i32) {
    check_next_is!(it, ind, Token::LPar);

    let mut fun_types: Vec<ASTNode> = Vec::new();
    match it.next() {
        Some(next) if next.token != Token::RPar => match parse_function_type(it, ind) {
            (Ok(fun_type), _) => fun_types.push(fun_type),
            err => return err
        }
        Some(next) => return (Err(TokenErr { expected: Token::RPar, actual: next.clone() }), ind),
        None => return (Err(EOFErr { expected: Token::RPar }), ind)
    }

    loop {
        match it.next() {
            Some(TokenPos { line: _, pos: _, token: Token::RPar }) => break,
            Some(TokenPos { line: _, pos: _, token: Token::Comma }) =>
                match parse_function_type(it, ind) {
                    (Ok(fun_type), _) => fun_types.push(fun_type),
                    err => return err
                }
            Some(next) =>
                return (Err(TokenErr { expected: Token::LPar, actual: next.clone() }), ind),
            None => return (Err(EOFErr { expected: Token::LPar }), ind)
        };
    }

    return (Ok(ASTNode::FunTuple(fun_types)), ind);
}

pub fn parse_function_anonymous(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                -> (ParseResult<ASTNode>, i32) {
    match parse_function_tuple(it, ind) {
        (Ok(tuple), ind) => {
            check_next_is!(it, ind, Token::To);
            match parse_expr_or_stmt(it, ind) {
                (Ok(body), ind) => (Ok(ASTNode::FunAnon(wrap!(tuple), wrap!(body))), ind),
                err => err
            }
        }
        err => err
    }
}
