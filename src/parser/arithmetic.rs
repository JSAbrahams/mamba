use crate::lexer::Token;
use crate::parser::ASTNode;
use crate::parser::parse_expression;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// arithmetic-expression    ::= term | unary-operator expression | term additive-operator expression
pub fn parse(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(_)) | Some(Token::Num(_)) | Some(Token::Str(_)) | Some(Token::Bool(_)) => {
            match parse_term(it, indent) {
                (Ok(term), new_indent) => {
                    match it.peek() {
                        Some(Token::Add) => {
                            it.next();
                            match parse_expression(it, new_indent) {
                                (Ok(expr), nnew_indent) =>
                                    (Ok(ASTNode::Add(Box::new(term), Box::new(expr))),
                                     nnew_indent),
                                err => err
                            }
                        }
                        Some(Token::Sub) => {
                            it.next();
                            match parse_expression(it, new_indent) {
                                (Ok(expr), nnew_indent) =>
                                    (Ok(ASTNode::Sub(Box::new(term), Box::new(expr))),
                                     nnew_indent),
                                err => err
                            }
                        }
                        _ => (Ok(term), new_indent)
                    }
                }
                err => err
            }
        }
        Some(Token::Not) => {
            it.next();
            match parse_expression(it, indent) {
                (Ok(expr), new_indent) => (Ok(ASTNode::Not(Box::new(expr))), new_indent),
                err => err
            }
        }
        Some(Token::Add) => {
            it.next();
            match parse_expression(it, indent) {
                (Ok(expr), new_indent) => (Ok(ASTNode::AddU(Box::new(expr))), new_indent),
                err => err
            }
        }
        Some(Token::Sub) => {
            it.next();
            match parse_expression(it, indent) {
                (Ok(expr), new_indent) => (Ok(ASTNode::SubU(Box::new(expr))), new_indent),
                err => err
            }
        }
        Some(_) => panic!("Expected arithmetic expression, but other token."),
        None => panic!("Expected arithmetic expression, but end of file.")
    };
}

// term                     ::= factor | factor multiclative-operator expression
fn parse_term(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(_)) | Some(Token::Str(_)) | Some(Token::Num(_)) | Some(Token::Bool(_)) =>
            match parse_factor(it, indent) {
                (Ok(factor), new_indent) => match it.peek() {
                    Some(Token::Mul) => {
                        it.next();
                        match parse_expression(it, new_indent) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Mul(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Div) => {
                        it.next();
                        match parse_expression(it, new_indent) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Div(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Mod) => {
                        it.next();
                        match parse_expression(it, new_indent) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Mod(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Pow) => {
                        it.next();
                        match parse_expression(it, new_indent) {
                            (Ok(expr), nnew_indent) =>
                                (Ok(ASTNode::Pow(Box::new(factor), Box::new(expr))),
                                 nnew_indent),
                            err => err
                        }
                    }
                    _ => (Ok(factor), new_indent)
                }
                err => err
            },
        Some(_) => panic!("Expected term, but other."),
        None => panic!("Expected term, but end of file.")
    };
}

// factor                   ::= constant | id
fn parse_factor(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    match it.next() {
        Some(Token::Id(id)) => (Ok(ASTNode::Id(id.to_string())), indent),
        Some(Token::Str(s)) => (Ok(ASTNode::Str(s.to_string())), indent),
        Some(Token::Num(num)) => (Ok(ASTNode::Num(*num)), indent),
        Some(Token::Bool(b)) => (Ok(ASTNode::Bool(*b)), indent),
        Some(_) => panic!("Expected factor, but other."),
        None => panic!("Expected factor, but end of file.")
    }
}
