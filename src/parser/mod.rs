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

// expression                  ::=
// "(" expression ")"
// | "return" expression
// | arithmetic-expression
// | control-flow-expression
fn parse_expression(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::LPar) => parse_bracket(it, indent),
        Some(Token::Ret) => parse_return(it, indent),
        Some(Token::Num(_)) | Some(Token::Id(_)) | Some(Token::Str(_)) | Some(Token::Not) |
        Some(Token::Add) | Some(Token::Sub) | Some(Token::Id(_)) => parse_arithmetic(it, indent),
        Some(Token::If) | Some(Token::When) | Some(Token::While) | Some(Token::Loop) =>
            parse_ctrl_flow(it, indent),

        _ => (Err("Token not recognized".to_string()), indent),
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

// arithmetic-expression       ::= term | unary-operator term | term additive-operator term
// term                        ::= factor | factor multiclative-operator factor
// factor                      ::= constant | id
fn parse_arithmetic(it: &mut Peekable<Iter<Token>>, indent: i32) -> (Result<ASTNode, String>, i32) {
    panic!("Not implemented")
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
