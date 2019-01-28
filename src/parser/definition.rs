use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_reassignment(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Assign);
    let right: Box<ASTNodePos> = get_or_err!(it, parse_expression, "reassignment");

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: right.en_line,
        en_pos: right.en_pos,
        node: ASTNode::ReAssign { left: Box::new(pre), right },
    });
}

pub fn parse_forward(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Forward);

    let mut forwarded: Vec<ASTNodePos> = Vec::new();
    let mut en_line = st_line;
    let mut en_pos = st_pos;

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::NL, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let def: ASTNodePos = get_or_err_direct!(it, parse_id, "forward");
                en_line = def.en_line;
                en_pos = def.en_pos;
                forwarded.push(def);
            }
            next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
        };
    }

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::Forward { forwarded },
    });
}

pub fn parse_definition(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Def);

    let private = it.peek().is_some() && it.peek().unwrap().token == Token::Private;
    if private { it.next(); }

    let definition = match it.peek() {
        _ => unimplemented!()
    };

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: st_line,
        en_pos: st_pos,
        node: ASTNode::Def { private, definition },
    });
}
