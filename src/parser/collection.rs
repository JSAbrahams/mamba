use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_collection(it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::LRBrack, .. }) => parse_tuple(it),
        Some(TokenPos { token: Token::LSBrack, .. }) => parse_list(it),
        Some(TokenPos { token: Token::LCBrack, .. }) => parse_set(it),

        Some(&next) => Err(CustomErr { expected: "collection".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "collection".to_string() })
    }
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LRBrack);

    let mut elements = Vec::new();
    match it.peek() {
        Some(TokenPos { token: Token::RRBrack, .. }) => (),
        _ => {
            let element = get_or_err_direct!(it, parse_expression, "tuple");
            elements.push(element);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RRBrack, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let element = get_or_err_direct!(it, parse_expression, "tuple");
                elements.push(element);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RRBrack);
    let (en_line, en_pos) = end_pos(it);

    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } });
}

fn parse_set(it: &mut TPIterator) -> ParseResult {
    unimplemented!()
}

fn parse_set_builder(before: ASTNode, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LCBrack);

    let set = get_or_err!(it, parse_expression, "set builder");
    check_next_is!(it, Token::Ver);

    let mut conditions = Vec::new();
    match it.peek() {
        Some(TokenPos { token: Token::RSBrack, .. }) => (),
        _ => {
            let condition = get_or_err_direct!(it, parse_expression, "tuple");
            conditions.push(condition);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RSBrack, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let condition = get_or_err_direct!(it, parse_expression, "tuple");
                conditions.push(condition);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RCBrack);
    let (en_line, en_pos) = end_pos(it);

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::SetBuilder { set, conditions },
    });
}

fn parse_map(first_key: ASTNode, it: &mut TPIterator) -> ParseResult {
    unimplemented!()
}

fn parse_list(it: &mut TPIterator) -> ParseResult {
    unimplemented!()
}
