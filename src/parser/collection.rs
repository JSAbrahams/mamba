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

macro_rules! get_or_err_zero_or_more { ($it:expr, $fun:path, $stop:path, $msg:expr) => {{
    let current = $it.peek().cloned();
    match $fun($it, $stop, $msg) {
        Ok(node) => Box::new(node),
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
        Some(TokenPos { token: Token::LCBrack, .. }) => parse_set(it),

        Some(&next) => Err(CustomErr { expected: "collection".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "collection".to_string() })
    }
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LRBrack);

    let elements: Box<ASTNodePos> = get_or_err_zero_or_more!(it, parse_zero_or_more_expr,
                                                             Token::RRBrack, "tuple");
    check_next_is!(it, Token::RRBrack);

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: elements.en_line,
        en_pos: elements.en_pos,
        node: ASTNode::Tuple { elements },
    });
}

fn parse_set(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LCBrack);

    unimplemented!()
}

fn parse_list(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LSBrack);

    unimplemented!()
}

fn parse_zero_or_more_expr(it: &mut TPIterator, close: Token, msg: &str) -> ParseResult {
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

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::ZeroOrMoreExpr { expressions },
    });
}
