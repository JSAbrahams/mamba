use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::call::parse_call;
use crate::parser::end_pos;
use crate::parser::expression::is_start_expression_exclude_unary;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_id(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    if let Some(TokenPos { token: Token::_Self, .. }) = it.peek() {
        let (en_line, en_pos) = start_pos(it);
        it.next();
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::_Self });
    }

    let (en_line, en_pos) = end_pos(it);
    let node = match it.next() {
        Some(TokenPos { token: Token::Init, .. }) => ASTNode::Init,
        Some(TokenPos { token: Token::Id(id), .. }) => ASTNode::Id { lit: id.to_string() },

        Some(next) =>
            return Err(CustomErr { expected: String::from("id"), actual: next.clone() }),
        None => return Err(EOFErr { expected: Token::Id(String::new()) })
    };

    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_id_maybe_call(it: &mut TPIterator) -> ParseResult {
    let id = match parse_id(it) {
        Ok(id) => id,
        err => return err
    };

    match it.peek() {
        Some(TokenPos { token: Token::Point, .. })
        | Some(TokenPos { token: Token::LRBrack, .. }) => parse_call(id, it),
        Some(&tp) if is_start_expression_exclude_unary(tp) => parse_call(id, it),
        _ => Ok(id)
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
                if it.peek().is_some() && it.peek().unwrap().token == Token::RSBrack {
                    break;
                }
                check_next_is!(it, Token::Comma);
            }
        }
    }

    check_next_is!(it, Token::RSBrack);
    Ok(generics)
}

pub fn parse_type(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let _type: ASTNodePos = match it.peek() {
        Some(TokenPos { token: Token::Id(_), .. }) => {
            let id: Box<ASTNodePos> = get_or_err!(it, parse_id, "type");
            let generics: Vec<ASTNodePos> = match it.peek() {
                Some(TokenPos { token: Token::LSBrack, .. }) =>
                    get_or_err_direct!(it, parse_generics, "type generic"),
                _ => vec![]
            };

            let (en_line, en_pos) = match generics.last() {
                Some(generic) => (generic.en_line, generic.en_pos),
                None => (id.en_line, id.en_pos)
            };

            let node = ASTNode::Type { id, generics };
            ASTNodePos { st_line, st_pos, en_line, en_pos, node }
        }
        _ => get_or_err_direct!(it, parse_type_tuple, "type")
    };

    match it.peek() {
        Some(TokenPos { token: Token::To, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_type, "type");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: right.en_line,
                en_pos: right.en_pos,
                node: ASTNode::TypeFun { _type: Box::from(_type), body: right }
            })
        }
        _ => Ok(_type)
    }
}

pub fn parse_conditions(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    check_next_is!(it, Token::When);
    match it.peek() {
        Some(TokenPos { token: Token::NL, .. }) => {
            it.next();
        }
        _ => return Ok(vec![get_or_err_direct!(it, parse_condition, "single condition")])
    }

    check_next_is!(it, Token::Indent);
    let mut conditions = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::Dedent => break,
            Token::NL => {
                it.next();
            }
            _ => conditions.push(get_or_err_direct!(it, parse_condition, "condition"))
        }
    }

    if it.peek().is_some() {
        check_next_is!(it, Token::Dedent);
    }
    Ok(conditions)
}

fn parse_condition(it: &mut TPIterator) -> ParseResult {
    let condition: Box<ASTNodePos> = get_or_err!(it, parse_expression, "condition");
    let _else: Option<Box<ASTNodePos>> = match it.peek() {
        Some(TokenPos { token: Token::Else, .. }) =>
            Some(get_or_err!(it, parse_expression, "condition else")),
        _ => None
    };

    let (en_line, en_pos) = match &_else {
        Some(ast_pos) => (ast_pos.en_line, ast_pos.en_pos),
        None => (condition.en_line, condition.en_pos)
    };

    let (st_line, st_pos) = (condition.st_line, condition.st_pos);
    let node = ASTNode::Condition { cond: condition, _else };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
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
    let node = ASTNode::TypeTup { types };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
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

    let (st_line, st_pos) = (id.st_line, id.st_pos);
    let node = ASTNode::IdType { id, _type };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}
