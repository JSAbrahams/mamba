use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::collection::parse_zero_or_more_expr;
use crate::parser::end_pos;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::Point, .. }) => parse_method_call(pre, it),
        Some(TokenPos { token: Token::DDoublePoint, .. }) => {
            it.next();
            parse_function_call(pre, it)
        }
        Some(_) => parse_function_call(pre, it),
        None => return Err(CustomEOFErr { expected: String::from("function arguments") })
    }
}

pub fn parse_function_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let brackets;
    if let Some(TokenPos { token: Token::LRBrack, .. }) = it.peek() {
        it.next();
        brackets = true;
    } else { brackets = false; }

    let args: Vec<ASTNodePos> = match parse_zero_or_more_expr(it, "function call") {
        Ok(a) => a,
        Err(err) => return Err(err)
    };

    let (en_line, en_pos) = end_pos(it);
    if brackets { check_next_is!(it, Token::RRBrack); }

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::FunCall { namespace: None, name: Box::from(pre), args },
    });
}

pub fn parse_method_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Point);

    let name: Box<ASTNodePos> = get_or_err!(it, parse_id, "method name");

    let brackets;
    if let Some(TokenPos { token: Token::LRBrack, .. }) = it.peek() {
        it.next();
        brackets = true;
    } else { brackets = false; }

    let args: Vec<ASTNodePos> = match parse_zero_or_more_expr(it, "function call") {
        Ok(a) => a,
        Err(err) => return Err(err)
    };

    let (en_line, en_pos) = end_pos(it);
    if brackets { check_next_is!(it, Token::RRBrack); }

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::MethodCall { object: Box::from(pre), name, args },
    });
}
