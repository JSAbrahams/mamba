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
    match it.next() {
        Some(actual @ TokenPos { line, pos, token }) if *token != Token::Point =>
            return (Err(TokenErr { expected: Token::Point, actual }), ind),
        None => return (Err(EOFErr { expected: Token::Point }), ind)
    }

    match it.next() {
        Some(TokenPos { line, pos, token: Token::Id(id) }) => match it.peek() {
            Some(TokenPos { line, pos, token: Token::LPar }) => match parse_tuple(it, ind) {
                (Ok(tuple), ind) => (Ok(ASTNode::FunCall(
                    wrap!(caller), wrap!(ASTNode::Id(id.to_string())), wrap!(tuple))), ind),
                err => err
            }
            Some(actual) => (Err(TokenErr { expected: Token::LPar, actual }), ind),
            None => (Err(EOFErr { expected: Token::LPar }), ind)
        }
        Some(actual) => (Err(TokenErr { expected: Token::Id(String::new()), actual }), ind),
        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    }
}

pub fn parse_function_call_direct(function: ASTNode, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                  -> (ParseResult<ASTNode>, i32) {
    match (function, it.peek()) {
        (ASTNode::Id(ref id), Some(TokenPos { line, pos, token: Token::LPar })) =>
            match parse_tuple(it, ind) {
                (Ok(tuple), ind) =>
                    (Ok(ASTNode::FunCallDirect(wrap!(ASTNode::Id(id.to_string())), wrap!(tuple))), ind),
                err => err
            }
        (_, Some(actual)) => (Err(TokenErr { expected: Token::Id(String::new()), actual }), ind),
        (_, _) => (Err(EOFErr { expected: Token::Id(String::new()) }), ind),
    }
}

pub fn parse_function_definition_body(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                      -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(actual) if *actual.token != Token::Fun =>
            return (Err(TokenErr { expected: Token::Fun, actual }), ind),
        None => return (Err(EOFErr { expected: Token::Fun }), ind)
    }

    return match it.next() {
        Some(TokenPos { line, pos, token: Token::Id(id) }) => match parse_args(it, ind) {
            (Ok(args), ind) => match it.next() {
                Some(TokenPos { line, pos, token: Token::To }) =>
                    match parse_expr_or_stmt(it, ind) {
                        (Ok(body), ind) => (Ok(ASTNode::FunDefNoRetType(
                            wrap!(ASTNode::Id(id.to_string())), args, wrap!(body))), ind),
                        err => err
                    }
                Some(TokenPos { line, pos, token: Token::DoublePoint }) =>
                    match parse_function_type(it, ind) {
                        (Ok(ret_type), ind) => match it.next() {
                            Some(TokenPos { line, pos, token: Token::To }) =>
                                match parse_expr_or_stmt(it, ind) {
                                    (Ok(body), ind) => (Ok(ASTNode::FunDef(
                                        wrap!(ASTNode::Id(id.to_string())),
                                        args,
                                        wrap!(ret_type),
                                        wrap!(body))), ind),
                                    err => err
                                }
                            Some(actual) => (Err(TokenErr { expected: Token::To, actual }), ind),
                            None => (Err(EOFErr { expected: Token::To }), ind)
                        },
                        err => err
                    }
                Some(actual) => (Err(TokenErr { expected: Token::DoublePoint, actual }), ind),
                None => (Err(EOFErr { expected: Token::DoublePoint }), ind)
            }
            (Err(err), ind) => (Err(err), ind)
        }

        Some(actual) => (Err(TokenErr { expected: Token::Id(String::new()), actual }), ind),
        None => (Err(EOFErr { expected: Token::Id(String::new()) }), ind)
    };
}

fn parse_args(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<Vec<ASTNode>>, i32) {
    match it.next() {
        Some(actual @ TokenPos { ref line, ref pos, token }) if *token != Token::LPar =>
            return (Err(TokenErr { expected: Token::LPar, actual }), ind),
        None => return (Err(EOFErr { expected: Token::LPar }), ind)
    }
    let mut args = Vec::new();
    if it.peek() != Some(&&TokenPos::RPar) {
        match parse_function_arg(it, ind) {
            (Ok(arg), _) => args.push(arg),
            (Err(err), ind) => return (Err(err), ind)
        }
    }

    loop {
        match it.next() {
            Some(TokenPos { line, pos, token: Token::RPar }) => break,
            Some(TokenPos { line, pos, token: Token::Comma }) =>
                match parse_function_arg(it, ind) {
                    (Ok(fun_type), _) => args.push(fun_type),
                    (Err(err), ind) => return (Err(err), ind)
                }

            Some(actual) => (Err(TokenErr { expected: Token::RPar, actual }), ind),
            None => (Err(EOFErr { expected: Token::RPar }), ind)
        };
    }

    (Ok(args), ind)
}

fn parse_function_arg(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match parse_function_type(it, ind) {
        (Ok(arg), ind) => match it.next() {
            Some(TokenPos { line, pos, token: Token::DoublePoint }) =>
                match parse_function_type(it, ind) {
                    (Ok(ty), ind) => (Ok(ASTNode::FunArg(wrap!(arg), wrap!(ty))), ind),
                    err => err
                }

            Some(actual) => (Err(TokenErr { expected: Token::DoublePoint, actual }), ind),
            None => (Err(EOFErr { expected: Token::DoublePoint }), ind)
        },
        err => err
    }
}

fn parse_function_type(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line, pos, token: Token::Id(id) }) =>
            next_and!(it, (Ok(ASTNode::Id(id.to_string())), ind)),
        Some(TokenPos { line, pos, token: Token::LPar }) => match parse_function_tuple(it, ind) {
            (Ok(tup), ind) => {
                match it.next() {
                    Some(actual @ TokenPos { line, pos, token }) if *token != Token::To =>
                        return (Err(TokenErr { expected: Token::To, actual }), ind),
                    None => return (Err(EOFErr { expected: Token::To }), ind)
                }

                match parse_function_type(it, ind) {
                    (Ok(fun_ty), ind) => (Ok(ASTNode::FunType(wrap!(tup), wrap!(fun_ty))), ind),
                    err => err
                }
            }
            err => err
        }
        Some(actual) => (Err(TokenErr { expected: Token::LPar, actual }), ind),
        None => (Err(EOFErr { expected: Token::LPar }), ind)
    };
}

fn parse_function_tuple(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(actual @ TokenPos { line, pos, token }) if *token != Token::LPar =>
            return (Err(TokenErr { expected: Token::LPar, actual }), ind),
        None => return (Err(EOFErr { expected: Token::LPar }), ind)
    }

    let mut fun_types = Vec::new();
    match it.next() {
        Some(TokenPos { ref line, ref pos, token }) if *token != Token::RPar =>
            match parse_function_type(it, ind) {
                (Ok(fun_type), _) => fun_types.push(fun_type),
                err => return err
            }
    }

    loop {
        match it.next() {
            Some(TokenPos { line, pos, token: Token::RPar }) => break,
            Some(TokenPos { line, pos, token: Token::Comma }) =>
                match parse_function_type(it, ind) {
                    (Ok(fun_type), _) => fun_types.push(fun_type),
                    err => return err
                }
            Some(actual) => return (Err(TokenErr { expected: Token::LPar, actual }), ind),
            None => return (Err(EOFErr { expected: Token::LPar }), ind)
        };
    }

    return (Ok(ASTNode::FunTuple(fun_types)), ind);
}

pub fn parse_function_anonymous(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                -> (ParseResult<ASTNode>, i32) {
    match parse_function_tuple(it, ind) {
        (Ok(tuple), ind) => {
            match it.next() {
                Some(actual @ TokenPos { line, pos, token }) if *token != Token::To =>
                    return (Err(TokenErr { expected: Token::To, actual }), ind),
                None => return (Err(EOFErr { expected: Token::To }), ind)
            }

            match parse_expr_or_stmt(it, ind) {
                (Ok(body), ind) => (Ok(ASTNode::FunAnon(wrap!(tuple), wrap!(body))), ind),
                err => err
            }
        }
        err => err
    }
}
