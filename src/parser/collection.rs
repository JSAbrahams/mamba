use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;
use crate::parser::expression::parse_expression;

pub fn parse_collection(it: &mut TPIterator) -> ParseResult {
    unimplemented!()
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LPar);

    let mut elements = Vec::new();
    match it.peek() {
        Some(TokenPos { token: Token::RPar, .. }) => (),
        _ => {
            let element = get_or_err_direct!(it, parse_expression, "tuple");
            elements.push(element);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RPar, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let element = get_or_err_direct!(it, parse_expression, "tuple");
                elements.push(element);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RPar);
    let (en_line, en_pos) = end_pos(it);

    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } });
}

pub fn parse_set(it: &mut TPIterator) -> ParseResult {
    unimplemented!()
}

fn parse_set_builder(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LBrack);

    let set = get_or_err!(it, parse_expression, "set builder");
    check_next_is!(it, Token::Ver);

    let mut conditions = Vec::new();
    match it.peek() {
        Some(TokenPos { token: Token::RBrack, .. }) => (),
        _ => {
            let condition = get_or_err_direct!(it, parse_expression, "tuple");
            conditions.push(condition);
        }
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RBrack, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let condition = get_or_err_direct!(it, parse_expression, "tuple");
                conditions.push(condition);
            }
            tp =>
                return Err(CustomErr { expected: "tuple element".to_string(), actual: tp.clone() })
        };
    }

    check_next_is!(it, Token::RBrack);
    let (en_line, en_pos) = end_pos(it);

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::SetBuilder { set, conditions },
    });
}

pub fn parse_list(it: &mut TPIterator) -> ParseResult {
    unimplemented!()
}

pub fn parse_map(it: &mut TPIterator) -> ParseResult {
    unimplemented!()
}
