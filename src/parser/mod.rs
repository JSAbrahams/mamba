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
    IfElse(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
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
    return parse_do(&mut input.iter().peekable(), 0).0;
}

fn parse_do(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    let mut nodes = Vec::new();
    let mut last_newline = false;
    let mut new_indent= indent;

    while let Some(t) = it.peek() {
        let start_ident = indent;
        match parse_recursive(it, indent) {
            (Ok(ast_node), this_indent) => {
                nodes_push!(nodes, ast_node);

                let this_newline =
                    it.peek().is_some() && **it.peek().unwrap() == Token::NL;

                if start_ident > this_indent || this_newline && last_newline {
                    break; // indentation decreased, or double newline, marking end of do block
                } else if this_indent > start_ident {
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

fn parse_recursive(it: &mut Peekable<Iter<Token>>, indent: i32)
                   -> (Result<ASTNode, String>, i32) {
    it.next();
    return (Ok(ASTNode::Print(Box::new(ASTNode::Str("hello world".to_string())))), indent);
}

#[cfg(test)]
mod parser_tests;
