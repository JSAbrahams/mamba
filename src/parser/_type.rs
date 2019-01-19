use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_id(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let (en_line, en_pos) = end_pos(it);

    match it.next() {
        Some(TokenPos { token: Token::Id(id), .. }) => Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line,
            en_pos,
            node: ASTNode::Id { id: id.to_string() },
        }),

        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}


pub fn parse_type(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let _type = match match it.peek() {
        Some(TokenPos { token: Token::Id(_), .. }) => parse_id(it),
        _ => parse_type_tuple(it)
    } {
        Ok(t) => t,
        err => return err
    };

    return match it.peek() {
        Some(TokenPos { token: Token::Assign, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_type, "type");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: right.en_line,
                en_pos: right.en_pos,
                node: ASTNode::TypeFun { left: Box::new(_type), right },
            })
        }
        _ => Ok(_type)
    };
}

pub fn parse_type_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Type);

    let id = get_or_err!(it, parse_id, "type definition");
    check_next_is!(it, Token::Assign);
    let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "type definition");

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: _type.en_line,
        en_pos: _type.en_pos,
        node: ASTNode::TypeDef { id, _type },
    });
}

pub fn parse_type_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LPar);

    let mut types: Vec<ASTNodePos> = Vec::new();
    let mut en_line = st_line;
    let mut en_pos = st_pos;

    if it.peek().is_some() && it.peek().unwrap().token != Token::RPar {
        let id = get_or_err_direct!(it, parse_id, "type tuple");
        types.push(id);
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RPar, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();

                let _type: ASTNodePos = get_or_err_direct!(it, parse_type, "type");
                en_line = _type.en_line;
                en_pos = _type.en_pos;
                types.push(_type);
            }
            next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
        };
    }

    check_next_is!(it, Token::RPar);
    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::TypeTup { types },
    });
}

pub fn parse_id_maybe_type(it: &mut TPIterator) -> ParseResult {
    let id: Box<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::LPar, .. }) =>
            get_or_err!(it, parse_type_tuple, "id maybe type"),
        _ => get_or_err!(it, parse_id, "id maybe type")
    };

    if let Some(TokenPos { token: Token::DoublePoint, .. }) = it.peek() {
        it.next();
        let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "id maybe type");
        return Ok(ASTNodePos {
            st_line: id.st_line,
            st_pos: id.st_pos,
            en_line: _type.en_line,
            en_pos: _type.en_pos,
            node: ASTNode::IdAndType { id, _type },
        });
    } else { return Ok(*id); }
}

pub fn parse_id_and_type(it: &mut TPIterator) -> ParseResult {
    let id: Box<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::LPar, .. }) =>
            get_or_err!(it, parse_type_tuple, "id and type"),
        _ => get_or_err!(it, parse_id, "id and type")
    };

    check_next_is!(it, Token::DDoublePoint);
    let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "id and type");

    return Ok(ASTNodePos {
        st_line: id.st_line,
        st_pos: id.st_pos,
        en_line: _type.en_line,
        en_pos: _type.en_pos,
        node: ASTNode::IdAndType { id, _type },
    });
}
