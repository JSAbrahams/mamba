use std::iter::Iterator;
use core::slice::Iter;
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

    LE(Box<ASTNode>, Box<ASTNode>),
    GE(Box<ASTNode>, Box<ASTNode>),
    LEQ(Box<ASTNode>, Box<ASTNode>),
    GEQ(Box<ASTNode>, Box<ASTNode>),

    Is(Box<ASTNode>, Box<ASTNode>),
    Equals(Box<ASTNode>, Box<ASTNode>),
    Not(Box<ASTNode>),
    And(Box<ASTNode>, Box<ASTNode>),
    Or(Box<ASTNode>, Box<ASTNode>),

    If(Box<ASTNode>, Box<ASTNode>),
    When(Box<ASTNode>, Box<Vec<ASTNode>>),

    Do(Vec<Box<ASTNode>>),

    While(Box<ASTNode>, Box<ASTNode>),
    Loop(Vec<ASTNode>),
    ExitLoop,
    ContinueLoop
}


pub fn parse(input: Vec<Token>) -> Result<ASTNode, String> {
    match parse_iterator(&mut input.iter()) {
        Ok(astNodes) => return Ok(ASTNode::Do(astNodes)),
        Err(e) => return Err(e)
    }
}

fn parse_iterator(input: &mut Iter<Token>) -> Result<Vec<Box<ASTNode>>, String> {
    let mut nodes = Vec::new();

    while let Some(t) = input.next() {
        match t {
            Token::Num(num) => nodes.push(Box::from(ASTNode::Num(*num))),
            _ => return Err("not implemented.".to_string())
        };
    }

    return Ok(nodes);
}

#[cfg(test)]
mod parser_tests;
