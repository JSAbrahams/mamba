use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse;
use crate::parser::expression_or_statement::parse_maybe_expression;
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


// function-def      ::= "fun" id "(" [ { function-arg "," } function-arg ] ")"
//                       [ "->" ( id | static-tuple | function-arg ) ] "is" expr-or-stmt
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
                Some(Token::To) => match parse_id_or_static_tuple_or_function_sig(it, ind) {
                    (Ok(ret_type), nnew_ind) => match it.next() {
                        Some(Token::Is) => match parse(it, nnew_ind) {
                            (Ok(body), nnnew_ind) =>
                                (Ok(ASTNode::FunDef(
                                    Box::new(ASTNode::Id(id.to_string())),
                                    args,
                                    Box::new(ret_type),
                                    Box::new(body))), nnnew_ind),
                            err => err
                        }

                        Some(t) => (Err(format!("Expected function 'is', but got {:?}.", t)), ind),
                        None => (Err("Expected function 'is', but end of file.".to_string()), ind)
                    },
                    err => err
                }

                Some(t) => (Err(format!("Expected either 'is' or function return type\
                    , but got {:?}.", t)), ind),
                None => (Err("Expected either 'is' or function return type\
                , but end of file.".to_string()), ind)
            }
            (Err(err), new_ind) => (Err(err), new_ind)
        }

        Some(t) => (Err(format!("Expected function name, but got {:?}.", t)), ind),
        None => (Err("Expected function name, but end of file.".to_string()), ind)
    };
}

// "(" [ { function-arg "," } function-arg ] ")"
fn parse_function_args(it: &mut Peekable<Iter<Token>>, ind: i32)
                       -> (Result<Vec<ASTNode>, String>, i32) {
    match it.next() {
        Some(Token::LPar) => {
            let mut args = Vec::new();

            while Some(&&Token::Comma) != it.peek()
                && Some(&&Token::RPar) != it.peek() {
                match parse_function_arg(it, ind) {
                    (Ok(function_arg), _) => args.push(function_arg),
                    (Err(err), ind) => return (Err(err), ind)
                }
            }

            if it.next() != Some(&Token::RPar) {
                (Err("Expected closing bracket after tuple.".to_string()), ind)
            } else {
                (Ok(args), ind)
            }
        }

        Some(t) =>
            (Err(format!("Expected opening bracket for arguemnts, but got {:?}.", t)), ind),
        None => (Err("Expected opening bracket for arguemnts, but end of file.".to_string()), ind)
    }
}

// function-arg  ::= id ":" ( id | static-tuple | function-arg )
fn parse_function_arg(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match it.next() {
        Some(Token::Id(arg)) => match it.next() {
            Some(Token::DoublePoint) => {
                it.next();
                match parse_id_or_static_tuple_or_function_sig(it, ind) {
                    (Ok(ty), new_ind) => (Ok(ASTNode::FunArg(
                        Box::new(ASTNode::Id(arg.to_string())), Box::new(ty))), ind),
                    err => err
                }
            }

            Some(t) => (Err(format!("Expected double point, but got {:?}.", t)), ind),
            None => (Err("Expected double point, but end of file.".to_string()), ind)
        }

        Some(t) => (Err(format!("Expected argument type, but got {:?}.", t)), ind),
        None => (Err("Expected function argument, but end of file.".to_string()), ind)
    }
}

// static-tuple ::= "(" ( id | static-tuple | function-sig )
//                  { "," ( id | static-tuple | function-sig ) } ")"
// function-sig ::= static-tuple "->" ( id | static-tuple )
fn parse_id_or_static_tuple_or_function_sig(it: &mut Peekable<Iter<Token>>, ind: i32)
                                            -> (Result<ASTNode, String>, i32) {
    return match it.next() {
        Some(Token::Id(id)) => (Ok(ASTNode::Id(id.to_string())), ind),
        Some(Token::LPar) => {
            let mut elements = Vec::new();
            loop {
                elements.push(match parse_id_or_static_tuple_or_function_sig(it, ind) {
                    (Ok(tup), new_ind) => match it.peek() {
                        Some(Token::To) => {
                            it.next();
                            match parse_id_or_static_tuple_or_function_sig(it, ind) {
                                (Ok(to), _) =>
                                    ASTNode::FunSig(Box::new(tup), Box::new(to)),
                                err => return err
                            }
                        }
                        _ => tup
                    }
                    err => return err
                });

                if Some(&&Token::Comma) == it.peek() {
                    it.next();
                } else { break; }
            }

            match it.next() {
                Some(Token::RPar) => (Ok(ASTNode::Tuple(elements)), ind),

                Some(t) => (Err(format!("Expected closing bracket, but got {:?}.", t)), ind),
                None => (Err("Expected closing bracket, but end of file.".to_string()), ind)
            }
        }

        Some(t) => (Err(format!("Expected opening bracket, but got {:?}.", t)), ind),
        None => (Err("Expected opening bracket, but end of file.".to_string()), ind)
    };
}
