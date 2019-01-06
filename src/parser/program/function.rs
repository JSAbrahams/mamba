use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse;
use crate::parser::expression_or_statement::parse_maybe_expression;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// function-call  ::= maybe-expr "." id tuple
pub fn parse_call(it: &mut Peekable<Iter<Token>>, ind: i32)
                  -> (Result<(ASTNode, Vec<ASTNode>), String>, i32) {
    return match it.next() {
        Some(Token::Id(fun_name)) => match it.next() {
            Some(Token::LPar) => match parse_maybe_expression(it, ind) {
                (Ok(expr_or_stmt), new_ind) => match it.next() {
                    Some(&Token::RPar) => (Ok((ASTNode::Id(fun_name.to_string()), Vec::new())),
                                           new_ind),
                    Some(&Token::Comma) => {
                        let mut args = Vec::new();
                        args.push(expr_or_stmt);

                        while Some(&&Token::Comma) != it.peek()
                            && Some(&&Token::RPar) != it.peek() {
                            match parse(it, ind) {
                                (Ok(arg), _) => args.push(arg),
                                (Err(err), _) => return (Err(err), new_ind)
                            }
                        }

                        if it.next() != Some(&Token::RPar) {
                            (Err("Expected closing bracket after tuple.".to_string()), new_ind)
                        } else {
                            (Ok((ASTNode::Id(fun_name.to_string()), args)), new_ind)
                        }
                    }
                    _ => (Err("Expected either closing bracket after expression or statement, or \
                    comma between tuple elements.".to_string()), new_ind)
                }
                (Err(err), new_ind) => (Err(err), new_ind)
            }
            Some(t) => (Err(format!("Expected opening bracket, but got: {:?}", t)), ind),
            None => (Err("Expected opening bracket, but end of file.".to_string()), ind)
        }
        Some(t) => (Err(format!("Expected function name, but got: {:?}", t)), ind),
        None => (Err("Expected function name, but end of file.".to_string()), ind)
    };
}


// function-def   ::= "fun" id "(" { function-arg } ")" [ "->" ( id | function-tuple ) ] "is"
//                    expr-or-stmt
pub fn parse_function_definition(it: &mut Peekable<Iter<Token>>, ind: i32)
                                 -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::Fun));

    return match it.next() {
        Some(Token::Id(id)) => match parse_function_args(it, ind) {
            (Ok(args), new_ind) => match it.next() {
                Some(Token::Is) => match parse(it, new_ind) {
                    (Ok(body), nnew_ind) =>
                        (Ok(ASTNode::FunDefNoRetType(Box::new(ASTNode::Id(id.to_string())),
                                                     args, Box::new(body))), nnew_ind),
                    err => err
                }
                Some(Token::To) => panic!("Not implemented"),

                Some(t) =>
                    (Err(format!("Expected either is or function return type\
                    , but got {:?}.", t)), ind),
                None => (Err("Expected either is or function return type\
                , but end of file.".to_string()), ind)
            }
            (Err(err), new_ind) => (Err(err), new_ind)
        }

        Some(t) => (Err(format!("Expected function name, but got {:?}.", t)), ind),
        None => (Err("Expected function name,  but end of file.".to_string()), ind)
    };
}

fn parse_function_args(it: &mut Peekable<Iter<Token>>, ind: i32)
                       -> (Result<Vec<ASTNode>, String>, i32) {
    panic!("Not implemented")
}

// function-arg   ::= ( id | function-tuple ) id
fn parse_function_arg(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match it.peek() {
        Some(Token::Id(arg_type)) => {
            it.next();
            match it.next() {
                Some(Token::Id(arg)) =>
                    (Ok(ASTNode::FunArg(Box::new(ASTNode::Id(arg_type.to_string())),
                                        Box::new(ASTNode::Id(arg.to_string())))), ind),
                Some(t) => (Err(format!("Expected identifier after type, but got {:?}.", t)), ind),
                None => (Err("Expected identifier after type, but end of file.".to_string()), ind)
            }
        }
        Some(Token::LPar) => match parse_function_tuple(it, ind) {
            (Ok(tuple), new_ind) => match it.next() {
                Some(Token::Id(arg)) =>
                    (Ok(ASTNode::FunArg(Box::new((tuple)),
                                        Box::new(ASTNode::Id(arg.to_string())))), ind),
                Some(t) => (Err(format!("Expected identifier after type, but got {:?}.", t)), ind),
                None => (Err("Expected identifier after type, but end of file.".to_string()), ind)
            }
            err => err
        }

        Some(t) => (Err(format!("Expected argument type, but got {:?}.", t)), ind),
        None => (Err("Expected function argument, but end of file.".to_string()), ind)
    }
}

// function-tuple ::= "(" ( id | function tuple ) { "," ( id | function-tuple ) } ")"
fn parse_function_tuple(it: &mut Peekable<Iter<Token>>, ind: i32)
                        -> (Result<ASTNode, String>, i32) {
    debug_assert_eq!(it.next(), Some(&Token::LPar));

    let mut elements = Vec::new();

    panic!("not implemented");

    return (Ok(ASTNode::Tuple(elements)), ind);
}