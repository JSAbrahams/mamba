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

macro_rules! get_zero_or_more { ($it:expr, $stop:path, $msg:expr) => {{
    let current = $it.peek().cloned();
    match parse_zero_or_more_expr($it, $stop, $msg) {
        Ok(node) => node,
        Err(err) => return match current {
            Some(tp) => Err(ParseErr { parsing: $msg.to_string(), cause: Box::new(err),
                                       position: Some(tp.clone()) }),
            None =>
                Err(ParseErr { parsing: $msg.to_string(), cause: Box::new(err), position: None })
        }
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

    let elements: Vec<ASTNodePos> = get_zero_or_more!(it, Token::RRBrack, "tuple");
    let (en_line, en_pos) = end_pos(it);
    check_next_is!(it, Token::RRBrack);

    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } });
}

fn parse_set_or_map(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LCBrack);

    let head = get_or_err!(it, parse_expression, "set or map");

    return match it.peek() {
        Some(TokenPos { token: Token::Ver, .. }) => {
            it.next();
            let conditions: Vec<ASTNodePos> = get_zero_or_more!(it, Token::RCBrack,
                                                                "set conditions");
            return Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: 0,
                en_pos: 0,
                node: ASTNode::SetBuilder { items: head, conditions },
            });
        }
        Some(TokenPos { token: Token::To, .. }) => {
            unimplemented!();
        }
        _ => {
            let tail: Vec<ASTNodePos> = get_zero_or_more!(it, Token::RSBrack, "list");
            let (en_line, en_pos) = end_pos(it);
            check_next_is!(it, Token::RSBrack);

            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Set { head, tail } })
        }
    };
}

fn parse_key_value(it: &mut TPIterator) -> ParseResult {
    let key = get_or_err!(it, parse_expression, "key");
    check_next_is!(it, Token::To);
    let value = get_or_err!(it, parse_expression, "value");

    return Ok(ASTNodePos {
        st_line: key.st_line,
        st_pos: key.st_pos,
        en_line: value.en_line,
        en_pos: value.en_pos,
        node: ASTNode::KeyValue { key, value },
    });
}

fn parse_list(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LSBrack);

    let head = get_or_err!(it, parse_expression, "list");

    if let Some(TokenPos { token: Token::Ver, .. }) = it.peek() {
        it.next();
        let conditions: Vec<ASTNodePos> = get_zero_or_more!(it, Token::RSBrack, "list conditions");
        return Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line: 0,
            en_pos: 0,
            node: ASTNode::ListBuilder { items: head, conditions },
        });
    }

    let tail: Vec<ASTNodePos> = get_zero_or_more!(it, Token::RSBrack, "list");
    let (en_line, en_pos) = end_pos(it);
    check_next_is!(it, Token::RSBrack);

    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::List { head, tail } });
}

fn parse_zero_or_more_expr(it: &mut TPIterator, close: Token, msg: &str)
                           -> ParseResult<Vec<ASTNodePos>> {
    let (st_line, st_pos) = start_pos(it);
    let mut expressions = Vec::new();
    let mut en_line = st_line;
    let mut en_pos = st_pos;

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: close, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let expression: ASTNodePos = get_or_err_direct!(it, parse_expression, msg);

                en_line = expression.en_line;
                en_pos = expression.en_pos;
                expressions.push(expression);
            }
            tp =>
                return Err(CustomErr { expected: msg.to_owned() + " element", actual: tp.clone() })
        };
    }

    return Ok(expressions);
}
