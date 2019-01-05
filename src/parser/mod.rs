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

    Do(Vec<ASTNode>),

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
    IsN(Box<ASTNode>, Box<ASTNode>),
    Eq(Box<ASTNode>, Box<ASTNode>),
    Neq(Box<ASTNode>, Box<ASTNode>),
    Not(Box<ASTNode>),
    And(Box<ASTNode>, Box<ASTNode>),
    Or(Box<ASTNode>, Box<ASTNode>),

    If(Box<ASTNode>, Box<ASTNode>),
    IfElse(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    Unless(Box<ASTNode>, Box<ASTNode>),
    UnlessElse(Box<ASTNode>, Box<ASTNode>, Box<ASTNode>),
    When(Box<ASTNode>, Vec<ASTNode>),
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
//            | control-flow | expression "<-" expression
pub fn parse_expression(it: &mut Peekable<Iter<Token>>, ind: i32)
    -> (Result<ASTNode, String>, i32) {
    match match it.peek() {
        Some(Token::LPar) => expression::parse_bracket(it, ind),
        Some(Token::Ret) => expression::parse_return(it, ind),
        Some(Token::Real(_)) | Some(Token::Int(_)) | Some(Token::ENum(_, _)) | Some(Token::Id(_)) |
        Some(Token::Str(_)) | Some(Token::Bool(_)) | Some(Token::Not) | Some(Token::Add) |
        Some(Token::Sub) => arithmetic::parse(it, ind),
        Some(Token::If) | Some(Token::When) | Some(Token::For) | Some(Token::While) |
        Some(Token::Loop) => control_flow::parse(it, ind),

        Some(t) => (Err(format!("Unexpected token while parsing expression: {:?}", t).to_string()),
                    ind),
        None => (Err("Unexpected end of file.".to_string()), ind)
    } {
        (Ok(l_expr), new_ind) => match it.peek() {
            Some(Token::Assign) => {
                it.next();
                match parse_expression(it, new_ind) {
                    (Ok(r_expr), nnew_ind) =>
                        (Ok(ASTNode::Assign(Box::new(l_expr), Box::new(r_expr))),
                         nnew_ind),
                    err => err
                }
            }
            Some(_) | None => (Ok(l_expr), new_ind)
        }
        err => err
    }
}

// statement ::= "print" expression | identifier | "donothing"
fn parse_statement(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::Let) | Some(Token::Mut) => identifier::parse(it, ind),
        Some(Token::Print) => statement::parse_print(it, ind),
        Some(Token::DoNothing) => (Ok(ASTNode::DoNothing), ind),

        Some(t) => (Err(format!("Unexpected token while parsing statement: {:?}", t).to_string()),
                    ind),
        None => (Err("Unexpected end of file.".to_string()), ind)
    };
}

// do-block ::= ( { ( expression | statement ) newline } | newline )
pub fn parse_do(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    if let Err(err) = check_ind(it, ind) { return (Err(err), ind); }
    let mut nodes = Vec::new();
    let mut is_prev_empty_line = false;

    while let Some(&t) = it.peek() {
        let (res, this_ind) = match t {
            Token::Print | Token::Mut | Token::Let | Token::DoNothing => parse_statement(it, ind),
            Token::NL => {
                if is_prev_empty_line { return (Err("Double empty line found.".to_string()), ind); }
                is_prev_empty_line = true;
                it.next();
                continue;
            }
            _ => parse_expression(it, ind)
        };

        match res {
            Ok(ast_node) => {
                nodes.push(ast_node);

                if this_ind < ind {
                    break;
                } else if this_ind > ind {
                    return (Err("Indentation unexpectedly increased.".to_string()), ind);
                }

                is_prev_empty_line = false;
                if it.peek() != None && Some(&Token::NL) != it.next() {
                    return (Err(format!("Expression or statement not followed by a newline: {:?}.",
                                        it.peek())), ind);
                }
                if let Err(err) = check_ind(it, ind) {
                    /* if end of file doesn't matter */
                    if it.peek().is_some() { return (Err(err), ind); }
                }
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
mod parser_test;
mod parser_lexer_test;
