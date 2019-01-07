use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse as parse_expr_or_stmt;
use crate::parser::expression_or_statement::parse_tuple;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

mod function;

// program ::= { module-import newline } { newline } { function-def newline { newline } }
//             [ do-block ]
pub fn parse(it: &mut Peekable<Iter<Token>>) -> Result<ASTNode, String> {
    let mut modules = Vec::new();
    let mut functions = Vec::new();

    while let Some(&t) = it.peek() {
        match t {
            Token::From => match parse_module_import(it, 0) {
                (Ok(module), _) => modules.push(module),
                (err, _) => return err
            }
            _ => break
        };

        if it.next() != Some(&Token::NL) {
            return Err("module import not followed by a newline.".to_string());
        }
    }

    while let Some(&t) = it.peek() {
        match t {
            Token::NL => it.next(),
            _ => break
        };
    }

    while let Some(&t) = it.peek() {
        match t {
            Token::NL => it.next(),
            _ => break
        };
    }

    while let Some(&t) = it.peek() {
        match t {
            Token::Fun => match function::parse_function_definition(it, 0) {
                (Ok(definition), _) => functions.push(definition),
                (err, _) => return err
            }
            _ => break
        };

        if it.next() != Some(&Token::NL) {
            return Err("Function definition not followed by a newline.".to_string());
        }

        while let Some(&t) = it.peek() {
            match t {
                Token::NL => it.next(),
                _ => break
            };
        }
    }


    return match parse_do(it, 0) {
        (Ok(do_block), _) => Ok(ASTNode::Program(modules, functions, Box::new(do_block))),
        (err, _) => err
    };
}

// module-import ::= "from" id ( "use" id | "useall" )
fn parse_module_import(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::From));

    match it.next() {
        Some(Token::Id(m)) => match it.next() {
            Some(Token::UseAll) =>
                (Ok(ASTNode::ModuleAll(Box::new(ASTNode::Id(m.to_string())))), ind),
            Some(Token::Use) => match it.next() {
                Some(Token::Id(p)) =>
                    (Ok(ASTNode::Module(Box::new(ASTNode::Id(m.to_string())),
                                        Box::new(ASTNode::Id(p.to_string())))), ind),

                Some(t) => (Err(format!("Expected module property name, but got {:?}.", t)), ind),
                None => (Err("Expected module property name, but end of file.".to_string()), ind)
            }

            Some(t) => (Err(format!("Expected use modifier, but got {:?}.", t)), ind),
            None => (Err("Expected use modifier, but end of file.".to_string()), ind)
        }

        Some(t) => (Err(format!("Expected module name, but got {:?}.", t)), ind),
        None => (Err("Expected module name, but end of file.".to_string()), ind)
    }
}

// function-call-dir ::= id tuple
pub fn parse_function_call_direct(function: ASTNode, it: &mut Peekable<Iter<Token>>, ind: i32)
                                  -> (Result<ASTNode, String>, i32) {
    match function {
        ASTNode::Id(id) => match it.peek() {
            Some(Token::LPar) => match parse_tuple(it, ind) {
                (Ok(tuple), new_ind) => (Ok(ASTNode::DirectFunCall(
                    Box::new(ASTNode::Id(id)),
                    Box::new(tuple),
                )), new_ind),
                err => err
            }

            Some(t) => (Err(format!("Expected opening bracket, but got {:?}.", t)), ind),
            None => (Err("Expected opening bracket, but end of file.".to_string()), ind)
        }

        t => (Err(format!("Expected function name, but got {:?}.", t)), ind)
    }
}

// function-call ::= maybe-expr "." id tuple
pub fn parse_function_call(caller: ASTNode, it: &mut Peekable<Iter<Token>>, ind: i32)
                           -> (Result<ASTNode, String>, i32) {
    match it.next() {
        Some(Token::Point) => match it.next() {
            Some(Token::Id(id)) => match it.peek() {
                Some(Token::LPar) => match parse_tuple(it, ind) {
                    (Ok(tuple), new_ind) => (Ok(ASTNode::FunCall(
                        Box::new(caller),
                        Box::new(ASTNode::Id(id.to_string())),
                        Box::new(tuple),
                    )), new_ind),
                    err => err
                }

                Some(t) => (Err(format!("Expected opening bracket, but got {:?}.", t)), ind),
                None => (Err("Expected opening bracket, but end of file.".to_string()), ind)
            }

            Some(t) => (Err(format!("Expected function name, but got {:?}.", t)), ind),
            None => (Err("Expected function name, but end of file.".to_string()), ind)
        }

        Some(t) => (Err(format!("Expected point, but got {:?}.", t)), ind),
        None => (Err("Expected function 'is', but end of file.".to_string()), ind)
    }
}

// do-block ::= ( { expr-or-stmt newline } | newline )
pub fn parse_do(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    let this_ind = util::ind_count(it);
    if this_ind > ind {
        return (Err(format!("Expected indentation of {}, was {}.", ind, this_ind)), this_ind);
    }

    let mut nodes = Vec::new();
    let mut is_prev_empty_line = false;

    while let Some(&t) = it.peek() {
        match *t {
            Token::NL if is_prev_empty_line => break,
            Token::NL => {
                is_prev_empty_line = true;
                it.next();
                continue;
            }
            _ => ()
        }

        let (res, this_ind) = parse_expr_or_stmt(it, ind);
        match res {
            Ok(ast_node) => {
                nodes.push(ast_node);

                is_prev_empty_line = false;
                if it.peek() != None && Some(&Token::NL) != it.next() {
                    return (Err(format!("Expression or statement not followed by a newline: {:?}.",
                                        it.peek())), ind);
                }

                let next_ind = util::ind_count(it);
                /* Indentation decrease marks end of do block */
                if next_ind < ind { break; };

                if next_ind > ind && it.peek().is_some() {
                    /* indentation increased unexpectedly */
                    return (Err(
                        format!("Indentation increased in do block from {} to {}.", ind, next_ind)),
                            ind);
                }
            }
            err => return (err, this_ind)
        }
    }

    return (Ok(ASTNode::Do(nodes)), ind - 1);
}
