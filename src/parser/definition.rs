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

pub fn parse_top_level_def(it: &mut TPIterator) -> ParseResult {
    let definition: Box<ASTNodePos> = get_or_err!(it, parse_definition, "class level definition");

    match it.peek() {
        Some(TokenPos { token: Token::Forward, .. }) => {
            let forward: Box<ASTNodePos> = get_or_err!(it, parse_forward, "class level forward");
            Ok(ASTNodePos {
                st_line: definition.st_line,
                st_pos: definition.st_pos,
                en_line: forward.en_line,
                en_pos: forward.en_pos,
                node: ASTNode::TopLevelDef { definition, forward: Some(forward) },
            })
        }
        _ => Ok(ASTNodePos {
            st_line: 0,
            st_pos: 0,
            en_line: 0,
            en_pos: 0,
            node: ASTNode::TopLevelDef { definition, forward: None },
        })
    }
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
    let empty_def: Box<ASTNodePos> = get_or_err!(it, parse_empty_def, "definition");

    return match it.peek() {
        Some(TokenPos { token: Token::Assign, .. }) => {
            it.next();
            let expression: Box<ASTNodePos> = get_or_err!(it, parse_expression, "definition body");

            Ok(ASTNodePos {
                st_line: empty_def.st_line,
                st_pos: empty_def.st_pos,
                en_line: 0,
                en_pos: 0,
                node: ASTNode::Def { empty_def, expression: Some(expression) },
            })
        }
        _ => Ok(ASTNodePos {
            st_line: empty_def.st_line,
            st_pos: empty_def.st_pos,
            en_line: empty_def.en_line,
            en_pos: empty_def.en_pos,
            node: ASTNode::Def { empty_def, expression: None },
        })
    };
}

pub fn parse_empty_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Def);

    let mut _mut = false;
    let mut private = false;
    let mut of_mut = false;

    if let Some(TokenPos { token: Token::Private, .. }) = it.peek() {
        it.next();
        private = true;
    }

    if let Some(TokenPos { token: Token::Mut, .. }) = it.peek() {
        it.next();
        _mut = true;
    }

    let id_maybe_type = get_or_err!(it, parse_id_maybe_type, "empty definition");

    if let Some(TokenPos { token: Token::OfMut, .. }) = it.peek() {
        it.next();
        of_mut = true;
    }

    let (en_line, en_pos) = end_pos(it);
    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::EmptyDef { _mut, private, of_mut, id_maybe_type },
    });
}
