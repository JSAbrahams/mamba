use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use super::lexer::Token;

#[macro_use]
macro_rules! next_and { ($it:expr, $stmt:stmt) => {{ $it.next(); $stmt }} }

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

    Do(Vec<ASTNode>),

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
pub fn parse_expression(it: &mut Peekable<Iter<Token>>, ind: i32)
                        -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::LPar) => expression::parse_bracket(it, ind),
        Some(Token::Ret) => expression::parse_return(it, ind),
        Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) | Some(Token::Id(_)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) | Some(Token::Not) | Some(Token::Add) |
        Some(Token::Sub) => arithmetic::parse(it, ind),
        Some(Token::If) | Some(Token::When) | Some(Token::While) | Some(Token::Loop) =>
            control_flow::parse(it, ind),

        Some(_) => panic!("Parser given token it does not recognize."),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}

// statement ::= "print" expression | identifier | "donothing"
fn parse_statement(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) | Some(Token::Mut) => identifier::parse(it, ind),
        Some(Token::Print) => statement::parse_print(it, ind),
        Some(Token::DoNothing) => (Ok(ASTNode::DoNothing), ind),

        Some(_) => panic!("Parser given token it does not recognize."),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}

// do-block ::= ( { ( expression | statement ) newline } | newline )
pub fn parse_do(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    let mut nodes = Vec::new();
    let mut is_last_nl = false;

    while let Some(&t) = it.peek() {
        if let Err(err) = check_ind(it, ind) { return (Err(err), ind); }

        let (res, this_ind) = match *t {
            Token::Print | Token::Mut | Token::Let | Token::DoNothing => parse_statement(it, ind),
            _ => parse_expression(it, ind)
        };

        match res {
            Ok(ast_node) => {
                nodes.push(ast_node);

                if it.peek() != None && Some(&Token::NL) != it.next() {
                    return (Err("Expression or statement not followed by a newline.".to_string()),
                            ind);
                }

                let is_next_nl = it.peek().is_some() && it.peek().unwrap() == &&Token::NL;

                if this_ind < ind && !is_last_nl {
                    return (Err("Indentation decreased without newline.".to_string()), ind);
                } else if this_ind > ind {
                    return (Err("Indentation unexpectedly increased.".to_string()), ind);
                } else if is_next_nl && is_last_nl {
                    return (Err("A double newline may not be used.".to_string()), ind);
                } else if this_ind < ind && is_last_nl {
                    break;
                }

                is_last_nl = is_next_nl;
            }
            err => return (err, this_ind)
        }
    }

    return (Ok(ASTNode::Do(nodes)), ind - 1);
}

pub fn check_ind(it: &mut Peekable<Iter<Token>>, ind: i32) -> Result<(), String> {
    for i in 0..ind {
        if it.next() != Some(&Token::Ind) {
            return Err(format!("Expected indentation level of {}, but was {}.", ind, i));
        }
    }
    Ok(())
}

// expression-or-do ::= ( expression | newline indent do-block )
pub fn parse_expression_or_do(it: &mut Peekable<Iter<Token>>, ind: i32)
                              -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::NL) => {
            it.next();
            parse_do(it, ind + 1)
        }
        Some(_) => parse_expression(it, ind),
        None => (Ok(ASTNode::DoNothing), ind)
    };
}

#[cfg(test)]
mod parser_tests;
