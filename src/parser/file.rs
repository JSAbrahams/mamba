use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_conditions;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::block::parse_statements;
use crate::parser::definition::parse_definition;
use crate::parser::end_pos;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::From);

    let id: Box<ASTNodePos> = get_or_err!(it, parse_id, "import id");

    let (_use, all) = match it.peek() {
        Some(TokenPos { token: Token::Use, .. }) => (
            {
                it.next();
                let mut ids: Vec<ASTNodePos> = Vec::new();
                ids.push(get_or_err_direct!(it, parse_id, "use"));
                while let Some(&t) = it.peek() {
                    match t.token {
                        Token::Comma => {
                            it.next();
                            ids.push(get_or_err_direct!(it, parse_id, "use"))
                        }
                        _ => break
                    }
                }
                ids
            },
            false
        ),
        Some(TokenPos { token: Token::UseAll, .. }) => (vec![], true),
        _ => (vec![], false)
    };

    let _as: Option<Box<ASTNodePos>> = match it.peek() {
        Some(TokenPos { token: Token::As, .. }) => {
            it.next();
            Some(get_or_err!(it, parse_id, "as"))
        }
        _ => None
    };

    // end pos will be of id if useall is used
    let (en_line, en_pos) = match (&_use.last(), &_as) {
        (_, Some(def)) => (def.en_line, def.en_pos),
        (Some(def), _) => (def.en_line, def.en_pos),
        (..) => (id.en_line, id.en_pos)
    };

    let node = ASTNode::Import { id, _use, all, _as };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_class_body(it: &mut TPIterator) -> ParseResult {
    let mut isa = Vec::new();
    if let Some(TokenPos { token: Token::IsA, .. }) = it.peek() {
        isa.push(get_or_err_direct!(it, parse_id, "generic"));
        while let Some(&t) = it.peek() {
            match t.token {
                Token::Comma => {
                    it.next();
                    isa.push(get_or_err_direct!(it, parse_id, "generic"));
                }
                _ => break
            }
        }
    }

    while it.peek().is_some() && it.peek().unwrap().token == Token::NL {
        it.next();
    }
    if it.peek().is_some() {
        check_next_is!(it, Token::Indent);
    }

    let mut definitions = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::NL => {
                it.next();
            }
            Token::Dedent => break,
            _ => definitions.push(get_or_err_direct!(it, parse_definition, "body"))
        }
    }

    if it.peek().is_some() {
        check_next_is!(it, Token::Dedent);
    }

    let (st_line, st_pos) = match (isa.first(), definitions.first()) {
        (_, Some(def)) => (def.st_line, def.st_pos),
        (Some(def), _) => (def.st_line, def.st_pos),
        _ => start_pos(it)
    };

    let (en_line, en_pos) = match (isa.last(), definitions.last()) {
        (_, Some(def)) => (def.en_line, def.en_pos),
        (Some(def), _) => (def.en_line, def.en_pos),
        _ => end_pos(it)
    };

    let node = ASTNode::Body { isa, definitions };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_stateless(it: &mut TPIterator) -> ParseResult {
    check_next_is!(it, Token::Stateless);
    let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "name");
    let body = get_or_err!(it, parse_class_body, "util");

    Ok(ASTNodePos {
        st_line: body.st_line,
        st_pos:  body.st_pos,
        en_line: body.en_line,
        en_pos:  body.en_pos,
        node:    ASTNode::Stateless { _type, body }
    })
}

pub fn parse_stateful(it: &mut TPIterator) -> ParseResult {
    check_next_is!(it, Token::Stateful);
    let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "name");
    let body: Box<ASTNodePos> = get_or_err!(it, parse_class_body, "class");

    Ok(ASTNodePos {
        st_line: body.st_line,
        st_pos:  body.st_pos,
        en_line: body.en_line,
        en_pos:  body.en_pos,
        node:    ASTNode::Stateful { _type, body }
    })
}

pub fn parse_script(it: &mut TPIterator) -> ParseResult {
    let statements: Vec<ASTNodePos> = get_or_err_direct!(it, parse_statements, "script");

    let (st_line, st_pos, en_line, en_pos) = match (statements.first(), statements.last()) {
        (Some(first), Some(last)) => (first.st_line, first.st_pos, last.en_line, last.en_pos),
        (..) => (0, 0, 0, 0)
    };

    let node = ASTNode::Script { statements };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_module(it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::Stateless, .. }) => parse_stateless(it),
        Some(TokenPos { token: Token::Stateful, .. }) => parse_stateful(it),
        _ => parse_script(it)
    }
}

pub fn parse_file(it: &mut TPIterator) -> ParseResult {
    let mut imports = Vec::new();
    let mut modules = Vec::new();
    let mut type_defs = Vec::new();

    while let Some(&t) = it.peek() {
        match t.token {
            Token::NL => {
                it.next();
            }
            Token::From => imports.push(get_or_err_direct!(it, parse_import, "import")),
            Token::Type =>
                type_defs.push(get_or_err_direct!(it, parse_type_def, "type definition")),
            _ => modules.push(get_or_err_direct!(it, parse_module, "module"))
        }
    }

    let node = ASTNode::File { imports, modules, type_defs };
    Ok(ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 0, node })
}

pub fn parse_type_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Type);
    let _type = get_or_err!(it, parse_type, "type definition");

    match it.peek() {
        Some(TokenPos { token: Token::IsA, .. }) => {
            check_next_is!(it, Token::IsA);
            let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "type definition");

            let conditions: Option<Vec<ASTNodePos>> = match it.peek() {
                Some(TokenPos { token: Token::When, .. }) =>
                    Some(get_or_err_direct!(it, parse_conditions, "type definition")),
                _ => None
            };

            let (en_line, en_pos) = (_type.en_line, _type.en_pos);
            let node = ASTNode::TypeAlias { _type, conditions };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        Some(_) => {
            let body: Box<ASTNodePos> = get_or_err!(it, parse_class_body, "type body");

            let (en_line, en_pos) = (body.en_line, body.en_pos);
            let node = ASTNode::TypeDef { _type, body: Some(body) };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        _ => Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line: _type.en_line,
            en_pos: _type.en_pos,
            node: ASTNode::TypeDef { _type, body: None }
        })
    }
}
