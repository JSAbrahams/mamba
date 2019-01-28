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
            node: ASTNode::Id { _self, lit: id.to_string() },
        }),

        Some(next) => Err(TokenErr { expected: Token::Id(String::new()), actual: next.clone() }),
        None => Err(EOFErr { expected: Token::Id(String::new()) })
    }
}

pub fn parse_type(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    return match it.peek() {
        Some(TokenPos { token: Token::Id(_), .. }) => {
            let id = get_or_err!(it, parse_id, "type");
            Ok(*id)
        }
        _ => {
            let _type = get_or_err!(it, parse_type_tuple, "type");
            match it.peek() {
                Some(TokenPos { token: Token::To, .. }) => {
                    it.next();
                    let right: Box<ASTNodePos> = get_or_err!(it, parse_type, "type");
                    Ok(ASTNodePos {
                        st_line,
                        st_pos,
                        en_line: right.en_line,
                        en_pos: right.en_pos,
                        node: ASTNode::TypeFun { left: _type, right },
                    })
                }
                _ => Ok(*_type)
            }
        }
    };
}

pub fn parse_range(it: &mut TPIterator) -> ParseResult {
    check_next_is!(it, Token::In);
    let from = get_or_err!(it, parse_id_or_lit, "from range");

    let inclusive;
    match it.next() {
        Some(TokenPos { token: Token::Range, .. }) => inclusive = true,
        Some(TokenPos { token: Token::RangeIncl, .. }) => inclusive = false,
        Some(tp) => return Err(TokenErr { expected: Token::Range, actual: tp.clone() }),
        None => return Err(EOFErr { expected: Token::Range })
    }

    let to = get_or_err!(it, parse_id_or_lit, "to range");

    return Ok(ASTNodePos {
        st_line: from.st_line,
        st_pos: from.st_pos,
        en_line: to.en_line,
        en_pos: to.en_pos,
        node: if inclusive { ASTNode::RangeIncl { from, to } } else { ASTNode::Range { from, to } },
    });
}

fn parse_id_or_lit(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let (en_line, en_pos) = end_pos(it);
    macro_rules! literal { ($factor: expr, $ast: ident) => {{
        it.next();
        ASTNodePos { st_line, st_pos, en_line: en_line, en_pos: en_pos,
                     node: ASTNode::$ast { lit: $factor } }
    }}}

    return Ok(match it.next() {
        Some(TokenPos { token: Token::Id(id), .. }) =>
            get_or_err_direct!(it, parse_id, "id or literal"),
        Some(TokenPos { token: Token::Real(real), .. }) => literal!(real.to_string(), Real),
        Some(TokenPos { token: Token::Int(int), .. }) => literal!(int.to_string(), Int),
        Some(TokenPos { token: Token::Bool(ref _bool), .. }) => literal!(*_bool, Bool),
        Some(TokenPos { token: Token::Str(str), .. }) => literal!(str.to_string(), Str),
        Some(TokenPos { token: Token::ENum(num, exp), .. }) => ASTNodePos {
            st_line,
            st_pos,
            en_line,
            en_pos,
            node: ASTNode::ENum { num: num.to_string(), exp: exp.to_string() },
        },
        Some(tp) => return Err(CustomErr { expected: "literal".to_string(), actual: tp.clone() }),
        None => return Err(CustomEOFErr { expected: "literal".to_string() })
    });
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
    check_next_is!(it, Token::LRBrack);

    let mut types: Vec<ASTNodePos> = Vec::new();
    let mut en_line = st_line;
    let mut en_pos = st_pos;

    if it.peek().is_some() && it.peek().unwrap().token != Token::RRBrack {
        let id = get_or_err_direct!(it, parse_id, "type tuple");
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
    let id: Box<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::LRBrack, .. }) =>
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
        Some(TokenPos { token: Token::LRBrack, .. }) =>
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
