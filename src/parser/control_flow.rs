use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::check_ind;
use crate::parser::parse_expression;
use crate::parser::parse_expression_or_do;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow ::= if | when | loop | while | for | "break" | "continue"
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::If) => parse_if(it, ind),
        Some(Token::When) => parse_when(it, ind),
        Some(Token::While) => parse_while(it, ind),
        Some(Token::Loop) => parse_loop(it, ind),
        Some(Token::For) => parse_for(it, ind),
        Some(Token::Break) => next_and!(it, (Ok(ASTNode::Break), ind)),
        Some(Token::Continue) => next_and!(it, (Ok(ASTNode::Continue), ind)),

        Some(_) => panic!("Expected control flow statement, but other token."),
        None => panic!("Expected control flow statement, but end of file.")
    };
}


// if ::= ( "if" | "unless" ) expression "then" expression-or-do [ [ newline ]
//        "else" expression-or-do ]
fn parse_if(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::If));
    match parse_expression(it, ind) {
        (Ok(cond), new_ind) => {
            if it.next() != Some(&Token::Then) {
                return (Err("'Then' keyword expected".to_string()), new_ind);
            }
            match parse_expression(it, new_ind) {
                (Ok(then), nnew_ind) => match it.peek() {
                    Some(Token::Else) => {
                        it.next();
                        match parse_expression(it, nnew_ind) {
                            (Ok(otherwise), nnnew_ind) =>
                                (Ok(ASTNode::IfElse(Box::new(cond), Box::new(then),
                                                    Box::new(otherwise))), nnnew_ind),
                            err => err
                        }
                    }
                    _ => (Ok(ASTNode::If(Box::new(cond), Box::new(then))), nnew_ind)
                }
                err => err
            }
        }
        err => err
    }
}

// when ::= "when" expression "is" newline indent { when-case }
fn parse_when(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::When));
    match parse_expression(it, ind) {
        (Ok(expr), new_ind) => {
            if it.next() != Some(&Token::Is) {
                return (Err("Expected 'is' after 'when' expression".to_string()), new_ind);
            } else if it.next() != Some(&Token::NL) {
                return (Err("Expected newline after 'is' in 'when' expression".to_string()),
                        new_ind);
            }

            match parse_when_cases(it, ind + 1) {
                (Ok(cases), _) => (Ok(ASTNode::When(Box::new(expr), cases)), ind),
                (Err(err), new_ind) => (Err(err), new_ind)
            }
        }
        err => err
    }
}

fn parse_when_cases(it: &mut Peekable<Iter<Token>>, ind: i32)
                    -> (Result<Vec<ASTNode>, String>, i32) {
    let mut when_cases = Vec::new();
    let mut is_prev_empty_line = false;

    loop {
        if Some(&&Token::NL) == it.peek() {
            if is_prev_empty_line { return (Err("Double newline found.".to_string()), ind); }
            it.next();
            is_prev_empty_line = true;
            continue;
        }

        if let Err(err) = check_ind(it, ind) { return (Err(err), ind); }

        let (res, this_ind) = parse_when_case(it, ind);
        if it.next() != Some(&Token::NL) {
            return (Err("Expected newline after 'when' case expression".to_string()), ind);
        }

        if this_ind < ind && !is_prev_empty_line {
            return (Err("Indentation decreased in 'when' expression.".to_string()), ind);
        } else if this_ind > ind {
            return (Err("Indentation increased in 'when' expression.".to_string()), ind);
        } else if this_ind < ind && is_prev_empty_line {
            break;
        }

        is_prev_empty_line = false;
        if it.peek() != None && Some(&Token::NL) != it.next() {
            return (Err(format!("Expression or statement not followed by a newline in 'while'.\
                     found: {:?}.", it.peek())), ind);
        }
        if let Err(err) = check_ind(it, ind) {
            /* if end of file doesn't matter */
            if it.peek().is_some() { return (Err(err), ind); }
        }

        match res {
            Ok(when_case) => when_cases.push(when_case),
            Err(err) => return (Err(err), this_ind)
        }
    }

    (Ok(when_cases), ind)
}

// when-case ::= expression "then" expression-or-do
fn parse_when_case(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    match parse_expression(it, ind) {
        (Ok(expr), new_ind) => {
            if it.next() != Some(&Token::Then) {
                return (Err("Expected 'then' after when case expression".to_string()), new_ind);
            }

            match parse_expression_or_do(it, ind) {
                (Ok(expr_or_do), nnew_ind) =>
                    (Ok(ASTNode::If(Box::new(expr), Box::new(expr_or_do))),
                     nnew_ind),
                err => err
            }
        }
        err => err
    }
}

// loop ::= "loop" expression-or-do
fn parse_loop(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::Loop));
    return match parse_expression_or_do(it, ind) {
        (Ok(expr_or_do), new_ind) => (Ok(ASTNode::Loop(Box::new(expr_or_do))), new_ind),
        err => err
    };
}

// while ::= "while" expression "do" expression-or-do
fn parse_while(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::While));
    return match parse_expression(it, ind) {
        (Ok(cond), new_ind) => {
            if it.next() != Some(&Token::Do) {
                return (Err("Expected 'do' after while conditional.".to_string()), new_ind);
            }

            match parse_expression_or_do(it, new_ind) {
                (Ok(expr_or_do), nnew_ind) =>
                    (Ok(ASTNode::While(Box::new(cond), Box::new(expr_or_do))),
                     nnew_ind),
                err => err
            }
        }
        err => err
    };
}

// for ::= "for" expression "in" expression "do" expression-or-do
fn parse_for(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::For));
    return match parse_expression(it, ind) {
        (Ok(expr), new_ind) => {
            if it.next() != Some(&Token::In) {
                return (Err("Expected 'in' after for expression".to_string()), new_ind);
            }

            match parse_expression(it, new_ind) {
                (Ok(col), nnew_ind) => {
                    if it.next() != Some(&Token::Do) {
                        return (Err("Expected 'do' after for collection".to_string()), new_ind);
                    }

                    match parse_expression_or_do(it, nnew_ind) {
                        (Ok(expr_or_do), nnnew_ind) =>
                            (Ok(ASTNode::For(Box::new(expr), Box::new(col),
                                             Box::new(expr_or_do))), nnnew_ind),
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}
