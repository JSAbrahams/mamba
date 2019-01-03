use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use crate::parser::parse_expression_or_do;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow            ::= if | when | loop | while | for | "break" | "continue"
pub fn parse(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::If) => parse_if(it, indent),
        Some(Token::When) => parse_when(it, indent),
        Some(Token::Loop) => parse_loop(it, indent),
        Some(Token::For) => parse_for(it, indent),
        Some(Token::Break) => {
            it.next();
            (Ok(ASTNode::Break), indent)
        }
        Some(Token::Continue) => {
            it.next();
            (Ok(ASTNode::Continue), indent)
        }

        Some(_) => panic!("Expected control flow statement, but other token."),
        None => panic!("Expected control flow statement, but end of file.")
    };
}


// if                      ::= ( "if" | "unless" ) expression "then" expression-or-do
// [ [ newline ] "else" expression-or-do ]
fn parse_if(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::If));
    match parse_expression(it, indent) {
        (Ok(cond), new_indent) => {
            if it.next() != Some(&Token::Then) {
                return (Err("'Then' keyword expected".to_string()), new_indent);
            }

            match parse_expression(it, new_indent) {
                (Ok(then), nnew_indent) => match it.peek() {
                    Some(Token::Else) => {
                        it.next();
                        match parse_expression(it, nnew_indent) {
                            (Ok(otherwise), nnnew_indent) =>
                                (Ok(ASTNode::IfElse(Box::new(cond), Box::new(then),
                                                    Box::new(otherwise))), nnnew_indent),
                            err => err
                        }
                    }
                    _ => (Ok(ASTNode::If(Box::new(cond), Box::new(then))), nnew_indent)
                }
                err => err
            }
        }
        err => err
    }
}


//when                    ::= "when" expression "is" newline indent { when-case }
fn parse_when(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::When));
    match parse_expression(it, indent) {
        (Ok(expr), new_indent) => {
            if it.next() != Some(&Token::Is) {
                return (Err("Expected 'is' after 'when' expression".to_string()), new_indent);
            }

            // TODO check indent
            let mut when_cases = Vec::new();
            let mut this_indent = indent;
            while this_indent >= indent {
                if this_indent > indent {
                    return (Err("Indentation unexpectedly increased in when case.".to_string()),
                            this_indent);
                }

                match parse_when_case(it, indent) {
                    (Ok(case), new_indent) => {
                        this_indent = new_indent;
                        when_cases.push(case)
                    }
                    err => return err
                }
            }

            (Ok(ASTNode::When(Box::new(expr), when_cases)), new_indent)
        }
        err => err
    }
}

//when-case               ::= expression "then" expression-or-do
fn parse_when_case(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    match parse_expression(it, indent) {
        (Ok(expr), new_indent) => {
            if it.next() != Some(&Token::Then) {
                return (Err("Expected 'then' after when case expression".to_string()), new_indent);
            }

            match parse_expression_or_do(it, indent) {
                (Ok(expr_or_do), nnew_indent) =>
                    (Ok(ASTNode::If(Box::new(expr), Box::new(expr_or_do))),
                     nnew_indent),
                err => err
            }
        }
        err => err
    }
}

//loop                    ::= "loop" expression-or-do
fn parse_loop(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::Loop));
    return match parse_expression_or_do(it, indent) {
        (Ok(expr_or_do), new_indent) => (Ok(ASTNode::Loop(Box::new(expr_or_do))), new_indent),
        err => err
    };
}

//while                   ::= "while" expression "do" expression-or-do
fn parse_while(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::While));
    return match parse_expression(it, indent) {
        (Ok(cond), new_indent) => {
            if it.next() != Some(&Token::Do) {
                return (Err("Expected 'do' after while conditional.".to_string()), new_indent);
            }

            match parse_expression_or_do(it, new_indent) {
                (Ok(expr_or_do), nnew_indent) =>
                    (Ok(ASTNode::While(Box::new(cond), Box::new(expr_or_do))),
                     nnew_indent),
                err => err
            }
        }
        err => err
    };
}

//for                     ::= "for" expression "in" expression "do" expression-or-do
fn parse_for(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::For));
    return match parse_expression(it, indent) {
        (Ok(expr), new_indent) => {
            if it.next() != Some(&Token::In) {
                return (Err("Expected 'in' after for expression".to_string()), new_indent);
            }

            match parse_expression(it, new_indent) {
                (Ok(col), nnew_indent) => {
                    if it.next() != Some(&Token::Do) {
                        return (Err("Expected 'do' after for collection".to_string()), new_indent);
                    }

                    match parse_expression_or_do(it, nnew_indent) {
                        (Ok(expr_or_do), nnnew_indent) =>
                            (Ok(ASTNode::For(Box::new(expr), Box::new(col),
                                             Box::new(expr_or_do))), nnnew_indent),
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}
