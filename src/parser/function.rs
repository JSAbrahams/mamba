use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::maybe_expr::parse_tuple;
use crate::parser::parse_result::ParseError;
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
// function-anon    ::= function-tuple "->' maybe-expr

pub fn parse_function_call(caller: ASTNode, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                           -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { line, pos, token }) if *token != Token::Point =>
            return (Err(ParseError::TokenError(*tp, Token::Point)), ind),
        None => return (Err(ParseError::EOFError(Token::Point)), ind)
    }

    match (it.next(), it.peek()) {
        (Some(TokenPos { line, pos, token: Token::Id(id) }),
            Some(TokenPos { line, pos, token: Token::LPar })) =>
            match parse_tuple(it, ind) {
                (Ok(tuple), ind) => (Ok(ASTNode::FunCall(
                    wrap!(caller), wrap!(ASTNode::Id(id.to_string())), wrap!(tuple))), ind),
                err => err
            }
        (_, Some(TokenPos { line, pos, token: Token::LPar })) =>
            (Err("Expected identifier.".to_string()), ind),
        (_, _) => (Err("Expected opening bracket.".to_string()), ind),
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
        (_, Some(TokenPos::LPar)) => (Err("Expected identifier.".to_string()), ind),
        (_, _) => (Err("Expected opening bracket.".to_string()), ind),
    }
}

pub fn parse_function_definition_body(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                      -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::Fun =>
            return (Err(ParseError::TokenError(*tp, Token::Fun)), ind),
        None => return (Err(ParseError::EOFError(Token::Fun)), ind)
    }

    return if let Some(TokenPos::Id(id)) = it.next() {
        match parse_args(it, ind) {
            (Ok(args), ind) => match it.next() {
                Some(TokenPos::To) => match parse_expr_or_stmt(it, ind) {
                    (Ok(body), ind) => (Ok(ASTNode::FunDefNoRetType(
                        wrap!(ASTNode::Id(id.to_string())), args, wrap!(body))), ind),
                    err => err
                }
                Some(TokenPos::DoublePoint) => match parse_function_type(it, ind) {
                    (Ok(ret_type), ind) => match it.next() {
                        Some(TokenPos::To) => match parse_expr_or_stmt(it, ind) {
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

fn parse_args(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::LPar =>
            return (Err(ParseError::TokenError(*tp, Token::LPar)), ind),
        None => return (Err(ParseError::EOFError(Token::LPar)), ind)
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
            Some(TokenPos { line, pos, token: Token::Comma }) =>
                match parse_function_arg(it, ind) {
                    (Ok(fun_type), _) => args.push(fun_type),
                    (Err(err), ind) => return (Err(err), ind)
                }
            Some(TokenPos { line, pos, token: Token::RPar }) => break,

            Some(_) | None => return (Err(
                "Expected closing bracket after function arguments".to_string()), ind)
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
            Some(_) | None => (Err("Expected double point after argument id.".to_string()), ind)
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
                    Some(tp @ TokenPos { line, pos, token }) if *token != Token::To =>
                        return (Err(ParseError::TokenError(*tp, Token::To)), ind),
                    None => return (Err(ParseError::EOFError(Token::To)), ind)
                }

                match parse_function_type(it, ind) {
                    (Ok(fun_ty), ind) => (Ok(ASTNode::FunType(wrap!(tup), wrap!(fun_ty))), ind),
                    err => err
                }
            }
            err => err
        }
        Some(tp) => (Err(ParseError::TokenError(**tp, Token::LPar)), ind),
        None => (Err(ParseError::EOFError(Token::LPar)), ind)
    };
}

fn parse_function_tuple(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::LPar =>
            return (Err(ParseError::TokenError(*tp, Token::LPar)), ind),
        None => return (Err(ParseError::EOFError(Token::LPar)), ind)
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
            Some(tp) => (Err(ParseError::TokenError(**tp, Token::LPar)), ind),
            None => (Err(ParseError::EOFError(Token::LPar)), ind)
        };
    }

    return (Ok(ASTNode::FunTuple(fun_types)), ind);
}

pub fn parse_function_anonymous(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                                -> (ParseResult<ASTNode>, i32) {
    match parse_function_tuple(it, ind) {
        (Ok(tuple), ind) => {
            match it.next() {
                Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::To =>
                    return (Err(ParseError::TokenError(*tp, Token::To)), ind),
                None => return (Err(ParseError::EOFError(Token::To)), ind)
            }

            match parse_expression(it, ind) {
                (Ok(body), ind) => (Ok(ASTNode::FunAnon(wrap!(tuple), wrap!(body))), ind),
                err => err
            }
        }
        err => err
    }
}
