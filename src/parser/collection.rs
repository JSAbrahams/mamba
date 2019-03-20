use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::expression::is_start_expression;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

macro_rules! get_zero_or_more {
    ($it:expr, $msg:expr) => {{
        match parse_zero_or_more_expr($it, $msg) {
            Ok(node) => node,
            Err(err) => return Err(err)
        }
    }};
}

pub fn parse_collection(it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::LRBrack, .. }) => parse_tuple(it),
        Some(TokenPos { token: Token::LSBrack, .. }) => parse_list(it),
        Some(TokenPos { token: Token::LCBrack, .. }) => parse_set(it),

        Some(&next) =>
            Err(CustomErr { expected: "collection".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "collection".to_string() })
    }
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LRBrack);

    let elements: Vec<ASTNodePos> = get_zero_or_more!(it, "tuple");
    let (en_line, en_pos) = end_pos(it);
    check_next_is!(it, Token::RRBrack);

    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } })
}

fn parse_set(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LCBrack);
    if let Some(TokenPos { token: Token::RCBrack, .. }) = it.peek() {
        let (en_line, en_pos) = start_pos(it);
        it.next();

        let node = ASTNode::Set { elements: vec![] };
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
    }

    let head = get_or_err_direct!(it, parse_expression, "set");

    match it.peek() {
        Some(TokenPos { token: Token::Ver, .. }) => {
            it.next();
            let conditions: Vec<ASTNodePos> = get_zero_or_more!(it, "set builder");
            let (en_line, en_pos) = end_pos(it);
            check_next_is!(it, Token::RCBrack);

            let node = ASTNode::SetBuilder { items: Box::from(head), conditions };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        _ => {
            if let Some(&t) = it.peek() {
                if t.token == Token::Comma {
                    it.next();
                }
            }

            let mut elements = vec![head];
            let tail: Vec<ASTNodePos> = get_zero_or_more!(it, "set");
            elements.extend(tail);

            let (en_line, en_pos) = end_pos(it);
            check_next_is!(it, Token::RCBrack);

            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Set { elements } })
        }
    }
}

fn parse_list(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LSBrack);
    if let Some(TokenPos { token: Token::RSBrack, .. }) = it.peek() {
        let (en_line, en_pos) = start_pos(it);
        it.next();

        let node = ASTNode::List { elements: vec![] };
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
    }

    let head = get_or_err_direct!(it, parse_expression, "list");

    if let Some(TokenPos { token: Token::Ver, .. }) = it.peek() {
        it.next();
        let conditions: Vec<ASTNodePos> = get_zero_or_more!(it, "list builder");
        let (en_line, en_pos) = end_pos(it);
        check_next_is!(it, Token::RSBrack);

        let node = ASTNode::ListBuilder { items: Box::from(head), conditions };
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
    }

    if let Some(&t) = it.peek() {
        if t.token == Token::Comma {
            it.next();
        }
    }
    let mut elements = vec![head];
    let tail: Vec<ASTNodePos> = get_zero_or_more!(it, "list");
    elements.extend(tail);

    let (en_line, en_pos) = end_pos(it);
    check_next_is!(it, Token::RSBrack);

    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::List { elements } })
}

pub fn parse_one_or_more_expr(it: &mut TPIterator, msg: &str) -> ParseResult<Vec<ASTNodePos>> {
    let mut expressions = Vec::new();
    let mut pos = 0;

    if let Some(&t) = it.peek() {
        if !is_start_expression(t) {
            return Err(CustomErr { expected: String::from("expression"), actual: t.clone() });
        }
    } else {
        return Err(CustomEOFErr { expected: String::from("expression") });
    }

    while let Some(&t) = it.peek() {
        if !is_start_expression(t) {
            break;
        }
        expressions.push(get_or_err_direct!(
            it,
            parse_expression,
            String::from(msg) + " (pos " + &pos.to_string() + ")"
        ));
        match it.peek() {
            Some(TokenPos { token: Token::Comma, .. }) => {
                it.next();
            }
            _ => continue
        }
        pos += 1;
    }

    Ok(expressions)
}

pub fn parse_zero_or_more_expr(it: &mut TPIterator, msg: &str) -> ParseResult<Vec<ASTNodePos>> {
    match it.peek() {
        Some(&t) if is_start_expression(t) => parse_one_or_more_expr(it, msg),
        _ => Ok(vec![])
    }
}
