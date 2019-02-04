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

pub fn parse_id(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let _self;
    if let Some(TokenPos { token: Token::_Self, .. }) = it.peek() {
        it.next();
        _self = true;
    } else { _self = false; }

    let (en_line, en_pos) = end_pos(it);
    match it.next() {
        Some(TokenPos { token: Token::Id(id), .. }) => Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line,
            en_pos,
            node: ASTNode::Id { lit: id.to_string() },
        }),

        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}

pub fn parse_generics(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    check_next_is!(it, Token::LSBrack);
    let mut generics = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::RSBrack => break,
            _ => {
                generics.push(get_or_err_direct!(it, parse_id, "generic parameter"));
                if it.peek().is_some() && it.peek().unwrap().token == Token::RSBrack { break; }
                check_next_is!(it, Token::Comma);
            }
        }
    }

    check_next_is!(it, Token::RSBrack);
    return Ok(generics);
}

pub fn parse_type(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    return match it.peek() {
        Some(TokenPos { token: Token::Id(_), .. }) => {
            let id = get_or_err!(it, parse_id, "type");
            let generics: Option<Vec<ASTNodePos>> = match it.peek() {
                Some(TokenPos { token: Token::LSBrack, .. }) =>
                    Some(get_or_err_direct!(it, parse_generics, "type generic")),
                _ => None
            };

            return Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: 0,
                en_pos: 0,
                node: ASTNode::Type { id, generics },
            });
        }
        _ => {
            let tuple = get_or_err!(it, parse_type_tuple, "type");
            match it.peek() {
                Some(TokenPos { token: Token::To, .. }) => {
                    it.next();
                    let right: Box<ASTNodePos> = get_or_err!(it, parse_type, "type");
                    Ok(ASTNodePos {
                        st_line,
                        st_pos,
                        en_line: right.en_line,
                        en_pos: right.en_pos,
                        node: ASTNode::TypeFun { left: tuple, right },
                    })
                }
                _ => Ok(*tuple)
            }
        }
    };
}

pub fn parse_conditions(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    check_next_is!(it, Token::When);
    check_next_is!(it, Token::NL);
    check_next_is!(it, Token::Indent);
    let mut conditions = Vec::new();

    while let Some(&t) = it.peek() {
        match t.token {
            Token::Dedent => break,
            _ => {
                let condition = get_or_err!(it, parse_expression, "condition");
                check_next_is!(it, Token::Else);
                let _else = get_or_err!(it, parse_expression, "condition else");

                conditions.push(ASTNodePos {
                    st_line: 0,
                    st_pos: 0,
                    en_line: 0,
                    en_pos: 0,
                    node: ASTNode::Condition { condition, _else },
                })
            }
        }
    }

    check_next_is!(it, Token::Dedent);
    return Ok(conditions);
}

pub fn parse_type_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Type);

    let id = get_or_err!(it, parse_id, "type definition");
    check_next_is!(it, Token::IsA);
    let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "type definition");

    let conditions: Option<Vec<ASTNodePos>> = match it.peek() {
        Some(TokenPos { token: Token::Where, .. }) =>
            Some(get_or_err_direct!(it, parse_conditions, "type definition")),
        _ => None
    };

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: _type.en_line,
        en_pos: _type.en_pos,
        node: ASTNode::TypeDef { id, _type, conditions },
    });
}

pub fn parse_type_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::LRBrack);

    let mut types: Vec<ASTNodePos> = Vec::new();
    let mut en_line = st_line;
    let mut en_pos = st_pos;

    if it.peek().is_some() && it.peek().unwrap().token != Token::RRBrack {
        let id = get_or_err_direct!(it, parse_type, "type tuple");
        types.push(id);
    }

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::RRBrack, .. } => break,
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

    check_next_is!(it, Token::RRBrack);
    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::TypeTup { types },
    });
}

pub fn parse_id_maybe_type(it: &mut TPIterator) -> ParseResult {
    let id: Box<ASTNodePos> = get_or_err!(it, parse_id, "id maybe type");

    let (en_line, en_pos, _type) = match it.peek() {
        Some(TokenPos { token: Token::DoublePoint, .. }) => {
            it.next();
            let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "id type");
            (_type.en_line, _type.en_pos, Some(_type))
        }
        _ => (id.en_line, id.en_pos, None)
    };

    return Ok(ASTNodePos {
        st_line: id.st_line,
        st_pos: id.st_pos,
        en_line,
        en_pos,
        node: ASTNode::TypeId { id, _type },
    });
}

pub fn parse_id_and_type(it: &mut TPIterator) -> ParseResult {
    let id: Box<ASTNodePos> = get_or_err!(it, parse_id, "id and type");

    check_next_is!(it, Token::DDoublePoint);
    let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "id and type");

    return Ok(ASTNodePos {
        st_line: id.st_line,
        st_pos: id.st_pos,
        en_line: _type.en_line,
        en_pos: _type.en_pos,
        node: ASTNode::TypeId { id, _type: Some(_type) },
    });
}
