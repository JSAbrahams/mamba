use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// function-def     ::= "fun" id "(" [ { function-arg "," } function-arg ] ")" [ ":" function-type ]
//                      "->" expr-or-stmt
pub fn parse_function_definition(it: &mut Peekable<Iter<Token>>, ind: i32)
                                 -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Fun));

    return if let Some(Token::Id(id)) = it.next() {
        match parse_args(it, ind) {
            (Ok(args), new_ind) => match it.next() {
                Some(Token::To) => match parse(it, new_ind) {
                    (Ok(body), nnew_ind) =>
                        (Ok(ASTNode::FunDefNoRetType(wrap!(ASTNode::Id(id.to_string())),
                                                     args, wrap!(body))), nnew_ind),
                    err => err
                }
                Some(Token::DoublePoint) => match parse_function_type(it, ind) {
                    (Ok(ret_type), nnew_ind) => match it.next() {
                        Some(Token::To) => match parse(it, nnew_ind) {
                            (Ok(body), nnnew_ind) => (Ok(ASTNode::FunDef(
                                wrap!(ASTNode::Id(id.to_string())),
                                args,
                                wrap!(ret_type),
                                wrap!(body))), nnnew_ind),
                            err => err
                        }
                        Some(_) | None => (Err("Expected function 'is'.".to_string()), ind)
                    },
                    err => err
                }
                Some(_) | None => (Err("Expected either 'is' or function return type.".to_string()),
                                   ind)
            }
            (Err(err), new_ind) => (Err(err), new_ind)
        }
    } else {
        (Err("Expected function name".to_string()), ind)
    };
}

// function-args ::= function-type ":" function-type [ "," function-args ]
fn parse_args(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<Vec<ASTNode>, String>, i32) {
    return if let Some(Token::LPar) = it.next() {
        let mut args = Vec::new();
        if it.peek() != Some(&&Token::RPar) {
            match parse_function_arg(it, ind) {
                (Ok(arg), _) => args.push(arg),
                (Err(err), new_ind) => return (Err(err), new_ind)
            }
        }

        loop {
            match it.next() {
                Some(Token::Comma) => match parse_function_arg(it, ind) {
                    (Ok(fun_type), _) => args.push(fun_type),
                    (Err(err), new_ind) => return (Err(err), new_ind)
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

// function-arg ::= function-type ":" function-type
fn parse_function_arg(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match parse_function_type(it, ind) {
        (Ok(arg), new_ind) => match it.next() {
            Some(Token::DoublePoint) => match parse_function_type(it, new_ind) {
                (Ok(ty), nnew_ind) => (Ok(ASTNode::FunArg(wrap!(arg), wrap!(ty))), nnew_ind),
                err => err
            }
            Some(_) | None => (Err("Expected double point after argument id.".to_string()), ind)
        },
        err => err
    }
}

// function-type ::= id | static-tuple | static-tuple "->" function-type
fn parse_function_type(it: &mut Peekable<Iter<Token>>, ind: i32)
                       -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(id)) => next_and!(it, (Ok(ASTNode::Id(id.to_string())), ind)),
        Some(Token::LPar) => match parse_static_tuple(it, ind) {
            (Ok(tup), new_ind) => if let Some(Token::To) = it.peek() {
                it.next();
                match parse_function_type(it, new_ind) {
                    (Ok(fun_ty), nnew_ind) =>
                        (Ok(ASTNode::FunType(wrap!(tup), wrap!(fun_ty))), nnew_ind),
                    err => err
                }
            } else { (Ok(tup), new_ind) }
            err => err
        }
        Some(_) | None => (Err("Expected function type.".to_string()), ind)
    };
}

// static-tuple ::= "(" [ function-type { "," function-type } ] ")"
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
            Some(Token::Comma) => match parse_function_type(it, ind) {
                (Ok(fun_type), _) => fun_types.push(fun_type),
                err => return err
            }
            Some(Token::RPar) => break,
            Some(_) | None => return (Err("Expected function type.".to_string()), ind)
        };
    }

    return (Ok(ASTNode::StaticTuple(fun_types)), ind);
}
