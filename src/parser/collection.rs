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

macro_rules! get_zero_or_more { ($it:expr, $msg:expr) => {{
    let current = $it.peek().cloned();
    match parse_zero_or_more_expr($it, $msg) {
        Ok(node) => node,
        Err(err) => return Err(err)
    }
}}}

pub fn parse_collection(it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::LRBrack, .. }) => parse_tuple(it),
        Some(TokenPos { token: Token::LSBrack, .. }) => parse_list(it),
        Some(TokenPos { token: Token::LCBrack, .. }) => parse_set_or_map(it),

        Some(&next) => Err(CustomErr { expected: "collection".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "collection".to_string() })
    }
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LRBrack);

    let elements: Vec<ASTNodePos> = get_zero_or_more!(it, "tuple");
    let (en_line, en_pos) = end_pos(it);
    check_next_is!(it, Token::RRBrack);

    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } });
}

fn parse_set_or_map(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LCBrack);
    if let Some(TokenPos { token: Token::RCBrack, .. }) = it.peek() {
        let (en_line, en_pos) = start_pos(it);
        it.next();
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::EmptySet });
    }

    let head = get_or_err!(it, parse_expression, "set or map");

    return match it.peek() {
        Some(TokenPos { token: Token::Ver, .. }) => {
            it.next();
            let conditions: Vec<ASTNodePos> = get_zero_or_more!(it, "set builder");
            return Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: 0,
                en_pos: 0,
                node: ASTNode::SetBuilder { items: head, conditions },
            });
        }
        Some(TokenPos { token: Token::To, .. }) => {
            let key_value = match parse_key_value(*head, it) {
                Ok(k_v) => k_v,
                err => return err
            };

            if let Some(TokenPos { token: Token::Ver, .. }) = it.peek() {
                it.next();
                let conditions: Vec<ASTNodePos> = get_zero_or_more!(it, "map builder");
                let (en_line, en_pos) = start_pos(it);
                check_next_is!(it, Token::RCBrack);
                return Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line,
                    en_pos,
                    node: ASTNode::MapBuilder {
                        key_value: Box::from(key_value),
                        conditions: Vec::new(),
                    },
                });
            } else {
                let tail: Vec<ASTNodePos> = match parse_zero_or_more_key_value(it, "map") {
                    Ok(t) => t,
                    Err(err) => return Err(err)
                };

                let (en_line, en_pos) = end_pos(it);
                check_next_is!(it, Token::RCBrack);
                return Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line,
                    en_pos,
                    node: ASTNode::Map { key_value: Box::from(key_value), tail },
                });
            }
        }
        _ => {
            let tail: Vec<ASTNodePos> = get_zero_or_more!(it, "set");
            let (en_line, en_pos) = end_pos(it);
            check_next_is!(it, Token::RCBrack);

            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Set { head, tail } })
        }
    };
}

fn parse_key_value(key: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    check_next_is!(it, Token::To);
    let value = get_or_err!(it, parse_expression, "value");

    return Ok(ASTNodePos {
        st_line: key.st_line,
        st_pos: key.st_pos,
        en_line: value.en_line,
        en_pos: value.en_pos,
        node: ASTNode::KeyValue { key: Box::from(key), value },
    });
}

fn parse_list(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LSBrack);
    if let Some(TokenPos { token: Token::RSBrack, .. }) = it.peek() {
        let (en_line, en_pos) = start_pos(it);
        it.next();
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::EmptyList });
    }

    let head = get_or_err!(it, parse_expression, "list");

    if let Some(TokenPos { token: Token::Ver, .. }) = it.peek() {
        it.next();
        let conditions: Vec<ASTNodePos> = get_zero_or_more!(it, "list builder");
        let (en_line, en_pos) = end_pos(it);
        check_next_is!(it, Token::RSBrack);

        return Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line,
            en_pos,
            node: ASTNode::ListBuilder { items: head, conditions },
        });
    }

    let tail: Vec<ASTNodePos> = get_zero_or_more!(it, "list");
    let (en_line, en_pos) = end_pos(it);
    check_next_is!(it, Token::RSBrack);

    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::List { head, tail } });
}

pub fn parse_zero_or_more_expr(it: &mut TPIterator, msg: &str) -> ParseResult<Vec<ASTNodePos>> {
    let (st_line, st_pos) = start_pos(it);
    let mut expressions = Vec::new();
    let mut pos = 0;

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RRBrack, .. } | TokenPos { token: Token::RSBrack, .. } |
            TokenPos { token: Token::RCBrack, .. } | TokenPos { token: Token::NL, .. } => break,
            TokenPos { token: Token::Comma, .. } => { it.next(); }
            tp => {
                let expression: ASTNodePos = get_or_err_direct!(it, parse_expression,
                                             String::from(msg) + " (pos "+ &pos.to_string() + ")");
                expressions.push(expression);
            }
        }
        pos += 1;
    }

    return Ok(expressions);
}

fn parse_zero_or_more_key_value(it: &mut TPIterator, msg: &str) -> ParseResult<Vec<ASTNodePos>> {
    let (st_line, st_pos) = start_pos(it);
    let mut expressions = Vec::new();
    let mut pos = 0;

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RRBrack, .. } | TokenPos { token: Token::RSBrack, .. } |
            TokenPos { token: Token::RCBrack, .. } => break,
            TokenPos { token: Token::Comma, .. } => { it.next(); }
            tp => {
                let key = get_or_err!(it, parse_expression,
                                      String::from(msg) + " (pos "+ &pos.to_string() + ")");
                check_next_is!(it, Token::To);
                let value = get_or_err!(it, parse_expression,
                                        String::from(msg) + " (pos "+ &pos.to_string() + ")");

                expressions.push(ASTNodePos {
                    st_line: key.st_line,
                    st_pos: key.st_pos,
                    en_line: value.en_line,
                    en_pos: value.en_pos,
                    node: ASTNode::KeyValue { key, value },
                });
            }
        }
        pos += 1;
    }

    return Ok(expressions);
}
