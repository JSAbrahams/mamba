use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// reassignment         ::= maybe-expr "<-" maybe-expr
// defer-declaration    ::= declaration [ "forward" id { "," id } ]
// declaration          ::= mutable-assign | immutable-assign
// mutable-declaration  ::= [ "mutable" ] immutable-assignment
// immutable-declaration::= definition "<-" maybe-expr
// definition           ::= "let" id [ ":" id ]

pub fn parse_reassignment(pre: ASTNode, it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                          -> ParseResult<ASTNode> {
    check_next_is!(it, Token::Assign);
    let (expr, ind) = get_or_err!(it, ind, parse_expression, "reassignment");
    return Ok((ASTNode::Assign(Box::new(pre), expr), ind));
}

pub fn parse_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                         -> ParseResult<ASTNode> {
    return match match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Let }) =>
            parse_immutable_declaration(it, ind),
        Some(TokenPos { line: _, pos: _, token: Token::Mut }) => parse_mutable_declaration(it, ind),

        Some(&next) => Err(CustomErr { expected: "declaration".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "declaration".to_string() })
    } {
        Ok((declaration, ind)) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::Forward }) => {
                let mut properties: Vec<ASTNode> = Vec::new();
                while let Some(t) = it.peek() {
                    match *t {
                        TokenPos { line: _, pos: _, token: Token::NL } => break,
                        TokenPos { line: _, pos: _, token: Token::Comma } => {
                            it.next();
                            let (property, _) = get_or_err_direct!(it, ind, parse_expression,
                                                                   "defer declaration");
                            properties.push(property);
                        }
                        next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
                    };
                }
                Ok((ASTNode::Defer(Box::new(declaration), properties), ind))
            }
            _ => Ok((declaration, ind))
        },
        err => err
    };
}

fn parse_mutable_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> ParseResult<ASTNode> {
    check_next_is!(it, Token::Mut);
    let (dec, ind) = get_or_err!(it, ind, parse_immutable_declaration, "immutable declaration");
    return Ok((ASTNode::Mut(dec), ind));
}

fn parse_immutable_declaration(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                               -> ParseResult<ASTNode> {
    let (let_id, ind) = get_or_err!(it, ind, parse_definition, "definition");
    check_next_is!(it, Token::Assign);
    let (expr, ind) = get_or_err!(it, ind, parse_expression, "definition");
    return Ok((ASTNode::Assign(let_id, expr), ind));
}

fn parse_definition(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    check_next_is!(it, Token::Let);
    match it.next() {
        Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) => match it.peek() {
            Some(TokenPos { line: _, pos: _, token: Token::DoublePoint }) =>
                match (it.next(), it.next()) {
                    (_, Some(TokenPos { line: _, pos: _, token: Token::Id(id) })) =>
                        Ok((ASTNode::Let(Box::new(ASTNode::Id(id.to_string()))), ind)),
                    (_, Some(next)) => Err(TokenErr {
                        expected: Token::Id(String::new()),
                        actual: next.clone(),
                    }),
                    (_, None) => Err(EOFErr { expected: Token::Id(String::new()) })
                }
            _ => Ok((ASTNode::Let(Box::new(ASTNode::Id(id.to_string()))), ind))
        }
        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}
