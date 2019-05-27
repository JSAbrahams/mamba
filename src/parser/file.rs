use crate::lexer::token::Token;
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
    it.eat(Token::From)?;

    let id = it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Id(_) => it.parse(&parse_id, "from id"),
            _ => Err(TokenErr { expected: Token::Id(String::new()), actual: token_pos.clone() })
        },
        EOFErr { expected: Token::Id(String::new()) }
    )?;

    let import = it.parse(&parse_import, "import")?;

    let (en_line, en_pos) = (import.en_line, import.en_pos);
    let node = ASTNode::FromImport { id, import };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Import)?;

    let mut import = Vec::new();
    // TODO what about newlines?
    it.while_not_token(Token::As, &mut |it, _| {
        import.push(*it.parse(&parse_id, "import id")?);
        it.eat_if(Token::Comma);
        Ok(())
    })?;

    let _as = it.parse_vec_if_token(
        Token::As,
        &|it| {
            let mut aliases = Vec::new();
            it.while_not_token(Token::NL, &mut |it, token_pos| match token_pos.token {
                Token::Id(_) => {
                    aliases.push(*it.parse(&parse_id, "import")?);
                    it.eat_if(Token::Comma);
                    Ok(())
                }
                _ => Err(TokenErr {
                    expected: Token::Id(String::new()),
                    actual:   token_pos.clone()
                })
            })?;
            Ok(aliases)
        },
        "as imports"
    )?;

    let (en_line, en_pos) = match (import.last(), _as.last()) {
        (_, Some(token_pos)) => (token_pos.en_line, token_pos.en_pos),
        (Some(token_pos), _) => (token_pos.en_line, token_pos.en_pos),
        (..) => (st_line, st_pos + Token::Import.len())
    };
    let node = ASTNode::Import { import, _as };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_class(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Class)?;
    let _type = it.parse(&parse_type, "name")?;

    let mut args = vec![];
    it.while_not_token(Token::LRBrack, &mut |it, token_pos| match token_pos.token {
        Token::Def => {
            args.push(*it.parse(&parse_fun_arg, "constructor arg")?);
            Ok(())
        }
        _ => Err(TokenErr { expected: Token::Def, actual: token_pos.clone() })
    })?;

    let parents = it.parse_vec_if_token(
        Token::IsA,
        &|it| {
            let mut parents = vec![];
            it.while_not_token(Token::NL, &mut |it, token_pos| match token_pos.token {
                Token::Id(_) => {
                    parents.push(*it.parse(&parse_parent, "parent")?);
                    Ok(())
                }
                _ => Err(TokenErr {
                    expected: Token::Id(String::new()),
                    actual:   token_pos.clone()
                })
            })?;
            Ok(parents)
        },
        "parents"
    )?;

    it.while_token(Token::NL, &mut |_, _| Ok(()))?;

    let body = it.parse(&parse_block, "class body")?;
    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::Class { _type, args, parents, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_parent(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    let id = it.parse(&parse_id, "parent id")?;
    let generics = it.parse_vec_if_token(Token::LSBrack, &parse_generics, "parent generics")?;
    let mut args = vec![];
    it.while_not_token(Token::LRBrack, &mut |it, _| {
        args.push(*it.parse(&parse_expression, "parent argument")?);
        Ok(())
    })?;

    let (en_line, en_pos) = match (generics.last(), args.last()) {
        (_, Some(tp)) => (tp.en_line, tp.en_pos),
        (Some(tp), _) => (tp.en_line, tp.en_pos),
        _ => (id.en_line, id.en_pos)
    };
    let node = ASTNode::Parent { id, generics, args };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_script(it: &mut TPIterator) -> ParseResult {
    let statements = it.parse_vec(&parse_statements, "script")?;

    let (st_line, st_pos, en_line, en_pos) = match (statements.first(), statements.last()) {
        (Some(first), Some(last)) => (first.st_line, first.st_pos, last.en_line, last.en_pos),
        (..) => (0, 0, 0, 0)
    };

    let node = ASTNode::Script { statements };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_module(it: &mut TPIterator) -> ParseResult {
    if it.peak_if_fn(&|token_pos| token_pos.token == Token::Class) {
        parse_class(it)
    } else {
        parse_script(it)
    }
}

pub fn parse_file(it: &mut TPIterator) -> ParseResult {
    let mut imports = Vec::new();
    let mut modules = Vec::new();
    let mut type_defs = Vec::new();

    let pure = it.eat_if(Token::Pure);

    it.while_fn(&|_| true, &mut |it, token_pos| match &token_pos.token {
        Token::NL => Ok(()),
        Token::Import => {
            imports.push(*it.parse(&parse_import, "import")?);
            Ok(())
        }
        Token::From => {
            imports.push(*it.parse(&parse_from_import, "from import")?);
            Ok(())
        }
        Token::Type => {
            type_defs.push(*it.parse(&parse_type_def, "type definition")?);
            Ok(())
        }
        Token::Comment(comment) => {
            let (st_line, st_pos) = it.start_pos()?;
            let (en_line, en_pos) = it.end_pos()?;
            let node = ASTNode::Comment { comment: comment.clone() };
            modules.push(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
            Ok(())
        }
        _ => {
            modules.push(*it.parse(&parse_module, "module")?);
            Ok(())
        }
    })?;

    let node = ASTNode::File { pure, imports, modules, type_defs };
    Ok(Box::from(ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 0, node }))
}

pub fn parse_type_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    it.eat(Token::Type)?;
    let _type = it.parse(&parse_type, "type definition")?;

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::IsA => {
                it.eat(Token::IsA)?;
                let _type = it.parse(&parse_type, "type definition")?;
                let conditions =
                    it.parse_vec_if_token(Token::When, &parse_conditions, "type definitions")?;

                let (en_line, en_pos) = (_type.en_line, _type.en_pos);
                let node = ASTNode::TypeAlias { _type: _type.clone(), conditions };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => {
                it.while_token(Token::NL, &mut |_, _| Ok(()))?;
                let body = it.parse(&parse_block, "type body")?;
                let (en_line, en_pos) = (body.en_line, body.en_pos);
                let node = ASTNode::TypeDef { _type: _type.clone(), body: Some(body) };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
        },
        {
            let (en_line, en_pos) = (_type.en_line, _type.en_pos);
            let node = ASTNode::TypeDef { _type: _type.clone(), body: None };
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        }
    )
}
