use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use super::lexer::Token;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum ASTNode {
    Id(String),
    Num(f64),
    Str(String),
    Bool(bool),

    Add(Box<ASTNode>, Box<ASTNode>),
    Sub(Box<ASTNode>, Box<ASTNode>),
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
    When(Box<ASTNode>, Box<Vec<ASTNode>>),

    Do(Vec<Box<ASTNode>>),

    While(Box<ASTNode>, Box<ASTNode>),
    Loop(Vec<ASTNode>),
    Break,
    Continue,

    Print(Box<ASTNode>),
}

#[macro_use]
macro_rules! nodes_push { ( $ nodes:expr, $ node: expr  ) => { $nodes.push(Box::from($node)) } }

pub fn parse(input: Vec<Token>) -> Result<ASTNode, String> {
    match parse_iterator(&mut input.iter().peekable()) {
        Ok(ast_nodes) => return Ok(ASTNode::Do(ast_nodes)),
        Err(e) => return Err(e)
    }
}

fn parse_iterator(input: &mut Peekable<Iter<Token>>) -> Result<Vec<Box<ASTNode>>, String> {
    let mut nodes = Vec::new();

    while let Some(t) = input.next() {
        match parse_recursive(&t, input) {
            Ok(ast_node) => nodes_push!(nodes, ast_node),
            Err(err) => return Err(err)
        }
    }

    return Ok(nodes);
}

fn parse_recursive(token: &Token, input: &mut Peekable<Iter<Token>>)
                   -> Result<ASTNode, String> {
    match token {
        Token::Num(num) => match input.peek() {
            Some(Token::Add) => return Ok(
                ASTNode::Add(Box::from(ASTNode::Num(*num)),
                             Box::from(parse_recursive(&Token::Add, input).unwrap()))),
            _ => return Ok(ASTNode::Num(*num))
        }

        Token::Add | Token::Sub | Token::Mul | Token::Div | Token::Mod | Token::Pow
        | Token::Le | Token::Leq | Token::Geq | Token::Ge | Token::Is | Token::IsN
        | Token::Eq | Token::NEq | Token::And | Token::Or =>
            return parse_recursive(input.next().unwrap(), input),

        _ => return Err("Not implemented".to_string())
    }
}

#[cfg(test)]
mod parser_tests;
