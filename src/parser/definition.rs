use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::_type::parse_type;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::maybe_expr::parse_expression;
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
        node: ASTNode::Assign { left: Box::new(pre), right },
    });
}

pub fn parse_defer_definition(it: &mut TPIterator) -> ParseResult {
    panic!("Not implemented")
}

pub fn parse_im_defer_definition(it: &mut TPIterator) -> ParseResult {
    panic!("Not implemeneted")
}

pub fn parse_definition(it: &mut TPIterator) -> ParseResult {
    let def_tok: TokenPos = check_next_is!(it, Token::Def);

    match it.peek() {
        Some(TokenPos { token: Token::Mut, .. }) => mutable_def(def_tok, it),
        _ => immutable_def(def_tok, it)
    }
}

pub fn parse_mutable_def(it: &mut TPIterator) -> ParseResult {
    let def_tok: TokenPos = check_next_is!(it, Token::Def);
    return mutable_def(def_tok, it);
}

pub fn parse_immutable_def(it: &mut TPIterator) -> ParseResult {
    let def_tok: TokenPos = check_next_is!(it, Token::Def);
    return immutable_def(def_tok, it);
}

fn mutable_def(def_tok: TokenPos, it: &mut TPIterator) -> ParseResult {
    let _mut: TokenPos = check_next_is!(it, Token::Mut);
    let id_maybe_type: Box<ASTNodePos> = get_or_err!(it, parse_id_maybe_type, "mutable definition");
    let of_mut;

    if let Some(TokenPos { token: Token::OfMut, .. }) = it.peek() {
        it.next();
        of_mut = true;
    } else { of_mut = false; }

    match it.peek() {
        Some(TokenPos { token: Token::Assign, .. }) => {
            it.next();
            let expr = get_or_err!(it, parse_expression, "mutable definition");
            Ok(ASTNodePos {
                st_line: def_tok.line,
                st_pos: def_tok.pos,
                en_line: id_maybe_type.en_line,
                en_pos: id_maybe_type.en_pos,
                node: ASTNode::Def { _mut: true, of_mut, id_maybe_type, expr },
            })
        }
        _ => Ok(ASTNodePos {
            st_line: def_tok.line,
            st_pos: def_tok.pos,
            en_line: id_maybe_type.en_line,
            en_pos: id_maybe_type.en_pos,
            node: ASTNode::EmptyDef { _mut: true, of_mut, id_maybe_type },
        })
    }
}

fn immutable_def(def_tok: TokenPos, it: &mut TPIterator) -> ParseResult {
    let id_maybe_type: Box<ASTNodePos> = get_or_err!(it, parse_id_maybe_type,
                                                     "immutable definition");
    let of_mut;

    if let Some(TokenPos { token: Token::OfMut, .. }) = it.peek() {
        it.next();
        of_mut = true;
    } else { of_mut = false; }

    match it.peek() {
        Some(TokenPos { token: Token::Assign, .. }) => {
            it.next();
            let expr = get_or_err!(it, parse_expression, "mutable definition");
            Ok(ASTNodePos {
                st_line: def_tok.line,
                st_pos: def_tok.pos,
                en_line: id_maybe_type.en_line,
                en_pos: id_maybe_type.en_pos,
                node: ASTNode::Def { _mut: false, of_mut, id_maybe_type, expr },
            })
        }
        _ => Ok(ASTNodePos {
            st_line: def_tok.line,
            st_pos: def_tok.pos,
            en_line: id_maybe_type.en_line,
            en_pos: id_maybe_type.en_pos,
            node: ASTNode::EmptyDef { _mut: false, of_mut, id_maybe_type },
        })
    }
}

fn parse_forward(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Forward);

    let mut defs: Vec<ASTNodePos> = Vec::new();
    let mut en_line = st_line;
    let mut en_pos = st_pos;

    while let Some(t) = it.peek() {
        match *t {
            TokenPos { token: Token::NL, .. } => break,
            TokenPos { token: Token::Comma, .. } => {
                it.next();
                let def: ASTNodePos = get_or_err_direct!(it, parse_id, "foward");
                en_line = def.en_line;
                en_pos = def.en_pos;
                defs.push(def);
            }
            next => return Err(TokenErr { expected: Token::Comma, actual: next.clone() })
        };
    }

    return Ok(defs);
}
