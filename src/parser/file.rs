use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_conditions;
use crate::parser::_type::parse_generics;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::block::parse_statements;
use crate::parser::definition::parse_fun_arg;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_from_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::From);

    let id = it.peek(
        &|token_pos| match token_pos.token {
            Token::Id(_) => it.parse(&parse_id, "from id"),
            _ => Err(TokenErr { expected: Token::Id(String::new()), actual: token_pos.clone() })
        },
        EOFErr { expected: Token::Id(String::new()) }
    )?;

    let import = it.parse(&parse_import, "import")?;

    let (en_line, en_pos) = it.end_pos()?;
    let node = ASTNode::FromImport { id, import };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Import);

    let mut import = Vec::new();
    // TODO what about newlines?
    it.while_some_and_not(Token::As, &|token_pos| {
        import.push(*it.parse(parse_id, "import id")?);
        it.eat_if(Token::Comma);
        Ok(())
    });

    let _as = if it.peek().is_some() && it.peek().unwrap().token == Token::As {
        it.next();
        let mut aliases = Vec::new();
        while let Some(tp) = it.peek() {
            match tp.token {
                Token::Comma => {
                    it.next();
                }
                Token::Id(_) => {
                    aliases.push(it.parse(parse_id, "import"));
                    if let Some(tp) = it.peek() {
                        if tp.token != Token::Comma && tp.token != Token::NL {
                            return Err(TokenErr {
                                expected: Token::RRBrack,
                                actual:   (*tp).clone()
                            });
                        }
                    }
                }
                _ => break
            }
        }
        aliases
    } else {
        vec![]
    };

    let (en_line, en_pos) = it.end_pos()?;
    let node = ASTNode::Import { import, _as };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_class(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Class);
    let _type = it.parse(parse_type, "name");

    let args: Vec<ASTNodePos> = if let Some(&TokenPos { token: Token::LRBrack, .. }) = it.peek() {
        it.next();
        let mut args = vec![];
        let mut pos = 0;
        while let Some(tp) = it.peek() {
            match tp.token {
                Token::Comma => {
                    it.next();
                }
                Token::RRBrack => break,
                _ => {
                    it.eat(Token::Def);
                    match parse_fun_arg(it, pos) {
                        Ok(arg) => {
                            args.push(arg);
                            pos += 1;
                            if let Some(tp) = it.peek() {
                                if tp.token != Token::Comma && tp.token != Token::RRBrack {
                                    return Err(TokenErr {
                                        expected: Token::RRBrack,
                                        actual:   (*tp).clone()
                                    });
                                }
                            }
                        }
                        Err(err) => return Err(err)
                    }
                }
            }
        }
        it.eat(Token::RRBrack);
        args
    } else {
        vec![]
    };

    let parents = if let Some(&TokenPos { token: Token::IsA, .. }) = it.peek() {
        it.next();
        let mut parents = vec![];
        while let Some(&tp) = it.peek() {
            match tp.token {
                Token::Id(_) => parents.push(it.parse(parse_parent, "parent")),
                _ => break
            }
        }
        parents
    } else {
        vec![]
    };

    while let Some(&TokenPos { token: Token::NL, .. }) = it.peek() {
        it.next();
    }

    let body = it.parse(parse_block, "class body");
    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::Class { _type, args, parents, body };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_parent(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    let id: Box<ASTNodePos> = it.parse(parse_id, "parent id");
    let generics = if let Some(&TokenPos { token: Token::LSBrack, .. }) = it.peek() {
        it.parse(parse_generics, "parent generics")
    } else {
        vec![]
    };
    let args = if let Some(&TokenPos { token: Token::LRBrack, .. }) = it.peek() {
        it.next();
        let mut args = vec![];
        while let Some(_) = it.peek() {
            args.push(it.parse(parse_expression, "parent argument"));
            match it.peek() {
                Some(TokenPos { token: Token::Comma, .. }) => {
                    it.next();
                }
                _ => break
            }
        }
        it.eat(Token::RRBrack);
        args
    } else {
        vec![]
    };

    let (en_line, en_pos) = match (generics.last(), args.last()) {
        (_, Some(tp)) => (tp.en_line, tp.en_pos),
        (Some(tp), _) => (tp.en_line, tp.en_pos),
        _ => (id.en_line, id.en_pos)
    };
    let node = ASTNode::Parent { id, generics, args };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_script(it: &mut TPIterator) -> ParseResult {
    let statements: Vec<ASTNodePos> = it.parse(parse_statements, "script");

    let (st_line, st_pos, en_line, en_pos) = match (statements.first(), statements.last()) {
        (Some(first), Some(last)) => (first.st_line, first.st_pos, last.en_line, last.en_pos),
        (..) => (0, 0, 0, 0)
    };

    let node = ASTNode::Script { statements };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_module(it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::Class, .. }) => parse_class(it),
        _ => parse_script(it)
    }
}

pub fn parse_file(it: &mut TPIterator) -> ParseResult {
    let mut imports = Vec::new();
    let mut modules = Vec::new();
    let mut type_defs = Vec::new();

    let pure = it.peek().is_some() && it.peek().unwrap().token == Token::Pure;
    if pure {
        it.next();
    }

    while let Some(&t) = it.peek() {
        match &t.token {
            Token::NL => {
                it.next();
            }
            Token::Import => imports.push(it.parse(parse_import, "import")),
            Token::From => imports.push(it.parse(parse_from_import, "from import")),
            Token::Type => type_defs.push(it.parse(parse_type_def, "type definition")),
            Token::Comment(comment) => {
                it.next();
                modules.push(ASTNodePos {
                    st_line: t.line,
                    st_pos:  t.pos,
                    en_line: t.line,
                    en_pos:  t.pos + comment.len() as i32,
                    node:    ASTNode::Comment { comment: comment.clone() }
                })
            }
            _ => modules.push(it.parse(parse_module, "module"))
        }
    }

    let node = ASTNode::File { pure, imports, modules, type_defs };
    Ok(ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 0, node })
}

pub fn parse_type_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    it.eat(Token::Type);
    let _type = it.parse(&parse_type, "type definition")?;

    it.peek_or(
        &|token_pos| match token_pos.token {
            Token::IsA => {
                it.eat(Token::IsA);
                let _type = it.parse(&parse_type, "type definition")?;
                let conditions =
                    it.parse_vec_if(Token::When, &parse_conditions, "type definitions")?;

                let (en_line, en_pos) = (_type.en_line, _type.en_pos);
                let node = ASTNode::TypeAlias { _type, conditions };
                Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
            }
            _ => {
                it.while_some_and(Token::NL, &|_| {});
                let body = it.parse(&parse_block, "type body")?;
                let (en_line, en_pos) = (body.en_line, body.en_pos);
                let node = ASTNode::TypeDef { _type, body: Some(body) };
                Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
            }
            _ => {
                let node = ASTNode::TypeDef { _type, body: None };
                Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line: _type.en_line,
                    en_pos: _type.en_pos,
                    node
                })
            }
        },
        {
            let node = ASTNode::TypeDef { _type, body: None };
            let (en_line, en_pos) = (_type.en_line, _type.en_pos);
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        }
    )
}
