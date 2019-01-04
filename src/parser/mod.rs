use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use super::lexer::Token;

#[macro_use]
macro_rules! nodes_push { ( $ nodes:expr, $ node: expr  ) => { $nodes.push(Box::from($node)) } }

mod arithmetic;
mod control_flow;
mod expression;
mod identifier;
mod statement;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum ASTNode {
    Id(String),
    Assign(Box<ASTNode>, Box<ASTNode>),
    Mut(Box<ASTNode>),

    Real(String),
    Int(String),
    ENum(String, String),
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
    DoNothing,
}

// program ::= do-block
pub fn parse(input: Vec<Token>) -> Result<ASTNode, String> {
    return parse_do(&mut input.iter().peekable(), 0).0;
}

// expression ::= "(" ( expression-or-do | newline do ) ")" | "return" expression | arithmetic
//            | control-flow
pub fn parse_expression(it: &mut Peekable<Iter<Token>>, indent: i32)
                        -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::LPar) => expression::parse_bracket(it, indent),
        Some(Token::Ret) => expression::parse_return(it, indent),
        Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) | Some(Token::Id(_)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) | Some(Token::Not) | Some(Token::Add) |
        Some(Token::Sub) => arithmetic::parse(it, indent),
        Some(Token::If) | Some(Token::When) | Some(Token::While) | Some(Token::Loop) =>
            control_flow::parse(it, indent),

        Some(_) => panic!("Parser given token it does not recognize."),
        None => (Err("Unexpected end of file.".to_string()), indent)
    };
}

// statement ::= "print" expression | identifier | "donothing"
fn parse_statement(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) | Some(Token::Mut) => identifier::parse(it, indent),
        Some(Token::Print) => statement::parse_print(it, indent),
        Some(Token::DoNothing) => (Ok(ASTNode::DoNothing), indent),

        Some(_) => panic!("Parser given token it does not recognize."),
        None => (Err("Unexpected end of file.".to_string()), indent)
    };
}

// do-block ::= ( { ( expression | statement ) newline } | newline )
pub fn parse_do(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
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

    return (Ok(ASTNode::Do(nodes)), indent - 1);
}

// expression-or-do ::= ( expression | newline indent do-block )
pub fn parse_expression_or_do(it: &mut Peekable<Iter<Token>>, indent: i32)
                              -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::NL) => {
            it.next();
            parse_do(it, indent + 1)
        }
        Some(_) => parse_expression(it, indent),
        None => (Ok(ASTNode::DoNothing), indent)
    }
}

#[cfg(test)]
mod parser_tests;
