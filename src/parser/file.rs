use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::block::parse_statements;
use crate::parser::definition::parse_definition;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::From);

    let id: Box<ASTNodePos> = get_or_err!(it, parse_id, "import id");

    let (_use, all) = match it.peek() {
        Some(TokenPos { token: Token::Use, .. }) =>
            ({
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
             }, false),
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
        (_, _) => (id.en_line, id.en_pos)
    };

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::Import { id, _use, all, _as },
    });
}

pub fn parse_class_body(it: &mut TPIterator) -> ParseResult {
    let id: Box<ASTNodePos> = get_or_err!(it, parse_id, "name");

    let mut generics = Vec::new();
    if let Some(TokenPos { token: Token::LSBrack, .. }) = it.peek() {
        generics.push(get_or_err_direct!(it, parse_id_maybe_type, "generic"));
        while let Some(&t) = it.peek() {
            match t.token {
                Token::Comma => {
                    it.next();
                    generics.push(get_or_err_direct!(it, parse_id_maybe_type, "generic"));
                }
                _ => break
            }
        }
        check_next_is!(it, Token:: RSBrack);
    }

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

    while it.peek().is_some() && it.peek().unwrap().token == Token::NL { it.next(); }
    let mut definitions = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::NL => continue,
            _ => definitions.push(get_or_err_direct!(it, parse_definition, "body"))
        }
    }

    let (en_line, en_pos) = match (generics.last(), isa.last(), definitions.last()) {
        (_, _, Some(def)) => (def.en_line, def.en_pos),
        (_, Some(def), _) => (def.en_line, def.en_pos),
        (Some(def), _, _) => (def.en_line, def.en_pos),
        (_, _, _) => (id.en_line, id.en_pos)
    };

    return Ok(ASTNodePos {
        st_line: id.st_line,
        st_pos: id.st_pos,
        en_line,
        en_pos,
        node: ASTNode::Body { id, generics, isa, definitions },
    });
}

pub fn parse_util(it: &mut TPIterator) -> ParseResult {
    check_next_is!(it, Token::Util);
    let body = get_or_err!(it, parse_class_body, "util");
    return Ok(ASTNodePos {
        st_line: body.st_line,
        st_pos: body.st_pos,
        en_line: body.en_line,
        en_pos: body.en_pos,
        node: ASTNode::Util { body },
    });
}

pub fn parse_class(it: &mut TPIterator) -> ParseResult {
    check_next_is!(it, Token::Class);
    let body: Box<ASTNodePos> = get_or_err!(it, parse_class_body, "class");
    return Ok(ASTNodePos {
        st_line: body.st_line,
        st_pos: body.st_pos,
        en_line: body.en_line,
        en_pos: body.en_pos,
        node: ASTNode::Class { body },
    });
}

pub fn parse_script(it: &mut TPIterator) -> ParseResult {
    let statements: Vec<ASTNodePos> = get_or_err_direct!(it, parse_statements, "script");

    let (st_line, st_pos, en_line, en_pos) = match (statements.first(), statements.last()) {
        (Some(first), Some(last)) => (first.st_line, first.st_pos, last.en_line, last.en_pos),
        (_, _) => (0, 0, 0, 0),
    };

    return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Script { statements } });
}

pub fn parse_module(it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::Util, .. }) => parse_util(it),
        Some(TokenPos { token: Token::Class, .. }) => parse_class(it),
        _ => parse_script(it)
    }
}

pub fn parse_file(it: &mut TPIterator) -> ParseResult {
    let mut imports = Vec::new();
    let mut modules = Vec::new();

    while let Some(&t) = it.peek() {
        match t.token {
            Token::NL => { it.next(); }
            Token::From => imports.push(get_or_err_direct!(it, parse_import, "import")),
            _ => modules.push(get_or_err_direct!(it, parse_module, "module"))
        }
    }

    return Ok(ASTNodePos {
        st_line: 0,
        st_pos: 0,
        en_line: 0,
        en_pos: 0,
        node: ASTNode::File { imports, modules },
    });
}
