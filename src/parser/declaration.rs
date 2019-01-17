use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::env;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_reassignment(pre: ASTNode, it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "reassignment");
    check_next_is!(it, Token::Assign);

    let right = get_or_err!(it, parse_expression, "reassignment");
    return Ok(ASTNode::Assign { left: Box::new(pre), right });
}

pub fn parse_declaration(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "declaration");

    return match match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Let }) => parse_immutable_declaration(it),
        Some(TokenPos { line: _, pos: _, token: Token::Mut }) => parse_mutable_declaration(it),

        Some(&next) => Err(CustomErr { expected: "declaration".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "declaration".to_string() })
    } {
        Ok(declaration) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::Forward }) => {
                let mut properties: Vec<ASTNode> = Vec::new();
                while let Some(t) = it.peek() {
                    match *t {
                        TokenPos { line: _, pos: _, token: Token::NL } => break,
                        TokenPos { line: _, pos: _, token: Token::Comma } => {
                            it.next();
                            let property = get_or_err_direct!(it, parse_expression,
                                                              "defer declaration");
                            properties.push(property);
                        }
                        next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
                    };
                }
                Ok(ASTNode::Defer { declaration: Box::new(declaration), properties })
            }
            _ => Ok(declaration)
        },
        err => err
    };
}

fn parse_mutable_declaration(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "mutable declaration");
    check_next_is!(it, Token::Mut);

    let decl = get_or_err!(it, parse_immutable_declaration, "immutable declaration");
    return Ok(ASTNode::Mut { decl });
}

fn parse_immutable_declaration(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "immutable declaration");

    let left = get_or_err!(it, parse_definition, "definition");
    check_next_is!(it, Token::Assign);
    let right = get_or_err!(it, parse_expression, "definition");
    return Ok(ASTNode::Assign { left, right });
}

fn parse_definition(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "definition");
    check_next_is!(it, Token::Let);

    match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::DoublePoint }) =>
                match (it.next(), it.next()) {
                    (_, Some(TokenPos { line: _, pos: _, token: Token::Id(id) })) =>
                        Ok(ASTNode::Let { id: id.to_string() }),
                    (_, Some(next)) => Err(TokenErr {
                        expected: Token::Id(String::new()),
                        actual: next.clone(),
                    }),
                    (_, None) => Err(EOFErr { expected: Token::Id(String::new()) })
                }
            _ => Ok(ASTNode::Let { id: id.to_string() })
        }
        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}
