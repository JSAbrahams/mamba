use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::expression_or_statement::parse;
use crate::parser::expression_or_statement::parse_maybe_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// function-call  ::= [...] "." id tuple
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
// function-arg   ::= ( id | function-tuple ) id
// function-tuple ::= "(" ( id | function-tuple ) { "," ( id | function-tuple ) } ")"