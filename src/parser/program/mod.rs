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
    match (parse_multiple(&Token::From, &parse_module_import, it),
           parse_multiple(&Token::Fun, &function::parse_function_definition, it),
           parse_do(it, 0)) {
        (Err(err), _, _) | (_, Err(err), _) | (_, _, (Err(err), _)) => Err(err),
        (Ok(modules), Ok(functions), (Ok(do_block), _)) =>
            Ok(ASTNode::Program(modules, functions, wrap!(do_block)))
    }
}

fn parse_multiple(token: &Token,
                  fun: &Fn(&mut Peekable<Iter<Token>>, i32) -> (Result<ASTNode, String>, i32),
                  it: &mut Peekable<Iter<Token>>) -> Result<Vec<ASTNode>, String> {
    let mut elements = Vec::new();
    while let Some(&t) = it.peek() {
        if token != t { break; }

        match fun(it, 0) {
            (Ok(element), _) => elements.push(element),
            (Err(err), _) => return Err(err)
        }

        if it.next() != Some(&Token::NL) { return Err("Newline expected.".to_string()); }

        while let Some(&t) = it.peek() {
            match t {
                Token::NL => it.next(),
                _ => break
            };
        }
    }

    return Ok(elements);
}

// module-import ::= "from" id ( "use" id | "useall" )
fn parse_module_import(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::From));

    match it.next() {
        Some(Token::Id(m)) => match it.next() {
            Some(Token::UseAll) =>
                (Ok(ASTNode::ModuleAll(wrap!(ASTNode::Id(m.to_string())))), ind),
            Some(Token::Use) => match it.next() {
                Some(Token::Id(p)) =>
                    (Ok(ASTNode::Module(wrap!(ASTNode::Id(m.to_string())),
                                        wrap!(ASTNode::Id(p.to_string())))), ind),

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
                (Ok(tuple), new_ind) => (Ok(ASTNode::DirectFunCall(wrap!(ASTNode::Id(id)), wrap!(tuple))),
                                         new_ind),
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
                        wrap!(caller), wrap!(ASTNode::Id(id.to_string())), wrap!(tuple),
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
        if *t == Token::NL && is_prev_empty_line { break; }
        if *t == Token::NL { next_and!(it, { is_prev_empty_line = true; continue; }) }

        match parse_expr_or_stmt(it, ind) {
            (Ok(ast_node), new_ind) => {
                is_prev_empty_line = false;
                nodes.push(ast_node);

                if it.peek() != None && Some(&Token::NL) != it.next() {
                    return (Err("Line was not followed by a newline".to_string()), new_ind);
                }

                let next_ind = util::ind_count(it);
                /* Indentation decrease marks end of do block */
                if next_ind < new_ind { break; };

                if next_ind > new_ind && it.peek().is_some() {
                    return (Err(format!("Indentation unexpectedly increased.")), new_ind);
                }
            }
            err => return err
        }
    }

    return (Ok(ASTNode::Do(nodes)), ind - 1);
}
