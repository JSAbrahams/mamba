use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use super::lexer::Token;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum ASTNode {
    Id(String),
    Assign(Box<ASTNode>, Box<ASTNode>),
    Mut(Box<ASTNode>),
    Num(f64),
    Str(String),
    Bool(bool),

    Add(Box<ASTNode>, Box<ASTNode>),
    AddU(Box<ASTNode>),
    Sub(Box<ASTNode>, Box<ASTNode>),
    SubU(Box<ASTNode>),
    Mul(Box<ASTNode>, Box<ASTNode>),
    Div(Box<ASTNode>, Box<ASTNode>),
    Mod(Box<ASTNode>, Box<ASTNode>),
    Pow(Box<ASTNode>, Box<ASTNode>),

    Le(Box<ASTNode>, Box<ASTNode>),
    Ge(Box<ASTNode>, Box<ASTNode>),
    Leq(Box<ASTNode>, Box<ASTNode>),
    Geq(Box<ASTNode>, Box<ASTNode>),

    Is(Box<ASTNode>, Box<ASTNode>),
    Eq(Box<ASTNode>, Box<ASTNode>),
    Not(Box<ASTNode>),
    And(Box<ASTNode>, Box<ASTNode>),
    Or(Box<ASTNode>, Box<ASTNode>),

    If(Box<ASTNode>, Box<ASTNode>),
    IfElse(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    When(Box<ASTNode>, Box<Vec<ASTNode>>),

    Do(Vec<Box<ASTNode>>),

    While(Box<ASTNode>, Box<ASTNode>),
    Loop(Vec<ASTNode>),
    Break,
    Continue,
    Return(Box<ASTNode>),

    Print(Box<ASTNode>),
}

#[macro_use]
macro_rules! nodes_push { ( $ nodes:expr, $ node: expr  ) => { $nodes.push(Box::from($node)) } }

// program                     ::= do-block
pub fn parse(input: Vec<Token>) -> Result<ASTNode, String> {
    return parse_do(&mut input.iter().peekable(), 0).0;
}

// do-block                    ::= ( { ( expression | statement ) newline } | newline )
fn parse_do(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    let mut nodes = Vec::new();
    let mut last_newline = false;
    let mut new_indent = indent;

    while let Some(t) = it.peek() {
        match parse_expression(it, indent) {
            (Ok(ast_node), this_indent) => {
                nodes_push!(nodes, ast_node);

                let this_newline =
                    it.peek().is_some() && **it.peek().unwrap() == Token::NL;

                if new_indent > this_indent || this_newline && last_newline {
                    break; // indentation decreased, or double newline, marking end of do block
                } else if this_indent > new_indent {
                    return (Err("Indentation increased in do block.".to_string()), indent);
                }

                last_newline = this_newline;
                new_indent = this_indent
            }
            err => return err
        }
    }

    return (Ok(ASTNode::Do(nodes)), new_indent);
}

// expression ::= "(" expression ")" | "return" expression | ari-expression | cntrl-flow-expression
fn parse_expression(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::LPar) => parse_bracket(it, indent),
        Some(Token::Ret) => parse_return(it, indent),
        Some(Token::Num(_)) | Some(Token::Id(_)) | Some(Token::Str(_)) | Some(Token::Bool(_)) |
        Some(Token::Not) | Some(Token::Add) | Some(Token::Sub) => parse_arithmetic(it, indent),
        Some(Token::If) | Some(Token::When) | Some(Token::While) | Some(Token::Loop) =>
            parse_ctrl_flow(it, indent),

        Some(_) => panic!("token not recognized"),
        None => (Err("Unexpected end of file.".to_string()), indent)
    };
}

fn parse_bracket(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::LPar));
    let (expr, new_indent) = parse_expression(it, indent);
    return match it.next() {
        Some(Token::RPar) => (expr, new_indent),
        Some(t) => (Err("Expecting closing bracket.".to_string()), new_indent),
        None => (Err("Expected closing bracket, but end of file.".to_string()), new_indent)
    };
}

fn parse_return(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::Ret));
    return match parse_expression(it, indent) {
        (Ok(expr), new_indent) => (Ok(ASTNode::Return(Box::new(expr))), new_indent),
        err => err
    };
}

// arithmetic-expression    ::= term | unary-operator expression | term additive-operator expression
fn parse_arithmetic(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Id(_)) | Some(Token::Num(_)) | Some(Token::Str(_)) | Some(Token::Bool(_)) => {
            match parse_term(it, indent) {
                (Ok(term), new_indent) => {
                    match it.peek() {
                        Some(Token::Add) => {
                            it.next();
                            match parse_expression(it, new_indent) {
                                (Ok(expr), nnew_indent) => (Ok(ASTNode::Add(Box::new(term), Box::new(expr))), nnew_indent),
                                err => err
                            }
                        }
                        Some(Token::Sub) => {
                            it.next();
                            match parse_expression(it, new_indent) {
                                (Ok(expr), nnew_indent) => (Ok(ASTNode::Sub(Box::new(term), Box::new(expr))), nnew_indent),
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
                            (Ok(expr), nnew_indent) => (Ok(ASTNode::Mul(Box::new(factor), Box::new(expr))), nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Div) => {
                        it.next();
                        match parse_expression(it, new_indent) {
                            (Ok(expr), nnew_indent) => (Ok(ASTNode::Div(Box::new(factor), Box::new(expr))), nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Mod) => {
                        it.next();
                        match parse_expression(it, new_indent) {
                            (Ok(expr), nnew_indent) => (Ok(ASTNode::Mod(Box::new(factor), Box::new(expr))), nnew_indent),
                            err => err
                        }
                    }
                    Some(Token::Pow) => {
                        it.next();
                        match parse_expression(it, new_indent) {
                            (Ok(expr), nnew_indent) => (Ok(ASTNode::Pow(Box::new(factor), Box::new(expr))), nnew_indent),
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
        Some(t) => panic!("Expected factor, but other."),
        None => panic!("Expected factor, but end of file.")
    }
}

// control-flow-expression     ::= if-expression | when-expression
// if-expression               ::= "if" expression "then"
// ( newline indent do-block-expression | expression ) [ newline ] "else"
// ( newline indent do-block-expression | expression )
// when-expression             ::=
// "when" expression newline { indent when-case } [ newline indent "else"
// ( newline indent do-block-expression | expression ) ]
// when-case                   ::=
// "equals" expression "then" ( newline indent do-block-expression | expression )
fn parse_ctrl_flow(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    panic!("Not implemented")
}

#[cfg(test)]
mod parser_tests;
