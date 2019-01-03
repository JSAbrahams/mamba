use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use super::lexer::Token;

mod control_flow;
mod identifier;
mod arithmetic;

// TODO create system to measure indents at correct locations

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
    When(Box<ASTNode>, Vec<ASTNode>),

    Do(Vec<Box<ASTNode>>),

    For(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    While(Box<ASTNode>, Box<ASTNode>),
    Loop(Box<ASTNode>),
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
        match t {
            Token::Let | Token::Mut | Token::Loop | Token::While | Token::Continue | Token::Break =>
                match parse_statement(it, indent) {
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
            _ => match parse_expression(it, indent) {
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
    }

    return (Ok(ASTNode::Do(nodes)), new_indent);
}

fn parse_expression_or_do(it: &mut Peekable<Iter<Token>>, indent: i32)
                          -> (Result<ASTNode, String>, i32) {
    (Err("not implemented".to_string()), indent)
}

// statement                   ::= "(" statement ")" | identifier
fn parse_statement(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) | Some(Token::Mut) => identifier::parse(it, indent),

        Some(_) => panic!("token not recognized"),
        None => (Err("Unexpected end of file.".to_string()), indent)
    };
}

// expression ::= "(" expression ")" | "return" expression | ari-expression | cntrl-flow-expression
fn parse_expression(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::LPar) => parse_bracket_expression(it, indent),
        Some(Token::Ret) => parse_return(it, indent),
        Some(Token::Num(_)) | Some(Token::Id(_)) | Some(Token::Str(_)) | Some(Token::Bool(_)) |
        Some(Token::Not) | Some(Token::Add) | Some(Token::Sub) => arithmetic::parse(it, indent),
        Some(Token::If) | Some(Token::When) | Some(Token::While) | Some(Token::Loop) =>
            control_flow::parse(it, indent),

        Some(_) => panic!("token not recognized"),
        None => (Err("Unexpected end of file.".to_string()), indent)
    };
}

fn parse_bracket_expression(it: &mut Peekable<Iter<Token>>, indent: i32)
                            -> (Result<ASTNode, String>, i32) {
    assert_eq!(it.next(), Some(&Token::LPar));
    let (expr, new_indent) = parse_expression(it, indent);
    return match it.next() {
        Some(Token::RPar) => (expr, new_indent),
        Some(_) => (Err("Expecting closing bracket.".to_string()), new_indent),
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

#[cfg(test)]
mod parser_tests;
