use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use super::lexer::Token;

#[macro_use]
macro_rules! next_and { ($it:expr, $stmt:stmt) => {{ $it.next(); $stmt }} }
macro_rules! postfix_op { ($it:expr, $ind:expr, $op:path, $pre:expr) => {{
    $it.next();
    match parse_expression_or_do($it, $ind) {
        (Ok(post), nnew_ind) => (Ok($op(Box::new($pre), Box::new(post))), nnew_ind),
        err => err
    }
}}}

mod expression;
mod statement;
mod util;

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

// do-block ::= ( { ( expression | statement ) newline } | newline )
pub fn parse_do(it: &mut Peekable<Iter<Token>>, ind: i32) -> (Result<ASTNode, String>, i32) {
    if let Err(err) = util::check_ind(it, ind) { return (Err(err), ind); }
    let mut nodes = Vec::new();
    let mut is_prev_empty_line = false;

    while let Some(&t) = it.peek() {
        match *t {
            Token::NL if is_prev_empty_line => break,
            Token::NL => {
                is_prev_empty_line = true;
                it.next();
                continue;
            }
            _ => ()
        }

        let (res, this_ind) = parse_statement_or_expr(it, ind);
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
                if let Err(err) = util::check_ind(it, ind) {
                    /* if end of file doesn't matter */
                    if it.peek().is_some() { return (Err(err), ind); }
                }
            }
            err => return (err, this_ind)
        }
    }

    return (Ok(ASTNode::Do(nodes)), ind - 1);
}

// expression-or-do ::= ( expression | newline indent do-block )
pub fn parse_expression_or_do(it: &mut Peekable<Iter<Token>>, ind: i32)
                              -> (Result<ASTNode, String>, i32) {
    return match it.peek() {
        Some(Token::NL) => {
            it.next();
            parse_do(it, ind + 1)
        }
        Some(_) => expression::parse(it, ind),
        None => (Ok(ASTNode::DoNothing), ind)
    };
}

// statement-or-expr ::= ( statement | expression ) | expression "<-" expression-or-do | postfix-if
// postfix-if        ::= ( statement-or-expr ) ( "if" | "unless" ) expression-or-do
fn parse_statement_or_expr(it: &mut Peekable<Iter<Token>>, ind: i32)
                           -> (Result<ASTNode, String>, i32) {
    return match match it.peek() {
        Some(Token::Let) | Some(Token::Mut) | Some(Token::Print) | Some(Token::DoNothing) |
        Some(Token::For) | Some(Token::While) | Some(Token::Loop) => statement::parse(it, ind),
        _ => expression::parse(it, ind)
    } {
        (Ok(pre), new_ind) => match it.peek() {
            Some(Token::Assign) => postfix_op!(it, new_ind, ASTNode::Assign, pre),
            Some(Token::If) => postfix_op!(it, new_ind, ASTNode::If, pre),
            Some(Token::Unless) => postfix_op!(it, new_ind, ASTNode::Unless, pre),
            Some(_) | None => (Ok(pre), new_ind)
        }
        err => err
    };
}

#[cfg(test)]
mod test;
