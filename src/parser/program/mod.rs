use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

mod function;

// function-call  ::= maybe-expr "." id tuple
pub fn parse_function_call(id: ASTNode, it: &mut Peekable<Iter<Token>>, ind: i32)
                           -> (Result<ASTNode, String>, i32) {
    match function::parse_call(it, ind) {
        (Ok((func, args)), new_ind) =>
            (Ok(ASTNode::FunCall(Box::new(id), Box::new(func), args)), new_ind),
        (Err(err), _) => return (Err(err), ind)
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

        let (res, this_ind) = parse(it, ind);
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