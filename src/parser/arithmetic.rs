use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// arithmetic-expression ::= term | unary-operator expression | term additive-operator expression
pub fn parse(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(_)) | Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) => {
            match parse_term(it, ind) {
                (Ok(term), new_ind) => {
                    match it.peek() {
                        Some(Token::Add) => {
                            it.next();
                            match parse_expression(it, new_ind) {
                                (Ok(expr), nnew_ind) =>
                                    (Ok(ASTNode::Add(Box::new(term), Box::new(expr))),
                                     nnew_ind),
                                err => err
                            }
                        }
                        Some(Token::Sub) => {
                            it.next();
                            match parse_expression(it, new_ind) {
                                (Ok(expr), nnew_ind) =>
                                    (Ok(ASTNode::Sub(Box::new(term), Box::new(expr))),
                                     nnew_ind),
                                err => err
                            }
                        }
                        _ => (Ok(term), new_ind)
                    }
                }
                err => err
            }
        }
        Some(Token::Not) => {
            it.next();
            match parse_expression(it, ind) {
                (Ok(expr), new_indent) => (Ok(ASTNode::Not(Box::new(expr))), new_indent),
                err => err
            }
        }
        Some(Token::Add) => {
            it.next();
            match parse_expression(it, ind) {
                (Ok(expr), new_indent) => (Ok(ASTNode::AddU(Box::new(expr))), new_indent),
                err => err
            }
        }
        Some(Token::Sub) => {
            it.next();
            match parse_expression(it, ind) {
                (Ok(expr), new_indent) => (Ok(ASTNode::SubU(Box::new(expr))), new_indent),
                err => err
            }
        }

        Some(_) => panic!("Expected arithmetic expression, but other token."),
        None => panic!("Expected arithmetic expression, but end of file.")
    };
}

// term ::= factor | factor multiclative-operator expression
fn parse_term(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(_)) | Some(Token::Str(_)) | Some(Token::Real(_)) | Some(Token::Int(_)) |
        Some(Token::ENum(_, _)) | Some(Token::Bool(_)) =>
            match parse_factor(it, ind) {
                (Ok(factor), new_ind) => match it.peek() {
                    Some(Token::Mul) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Mul(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Div) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Div(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Pow) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Pow(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Mod) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Mod(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Eq) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Eq(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Is) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Is(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::IsN) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Not(Box::new(ASTNode::Is(Box::new(factor), Box::new(expr))))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Neq) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Not(Box::new(ASTNode::Eq(Box::new(factor), Box::new(expr))))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Ge) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Ge(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Geq) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Geq(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Le) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Le(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Leq) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Leq(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::And) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::And(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Or) => {
                        it.next();
                        match parse_expression(it, new_ind) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Or(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    _ => (Ok(factor), new_ind)
                }
                err => err
            },

        Some(_) => panic!("Expected term, but other."),
        None => panic!("Expected term, but end of file.")
    };
}


// factor ::= constant | id
fn parse_factor(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return (match it.next() {
        Some(Token::Id(id)) => Ok(ASTNode::Id(id.to_string())),
        Some(Token::Str(string)) => Ok(ASTNode::Str(string.to_string())),
        Some(Token::Real(real)) => Ok(ASTNode::Real(real.to_string())),
        Some(Token::Int(int)) => Ok(ASTNode::Int(int.to_string())),
        Some(Token::ENum(num, exp)) => Ok(ASTNode::ENum(num.to_string(), exp.to_string())),
        Some(Token::Bool(boolean)) => Ok(ASTNode::Bool(*boolean)),

        Some(_) => panic!("Expected factor, but other."),
        None => panic!("Expected factor, but end of file.")
    }, ind);
}
