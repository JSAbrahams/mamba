use crate::lexer::token::Token;
use crate::parser::_type::parse_conditions;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::block::parse_statements;
use crate::parser::class::parse_class;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_from_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::From, "from import")?;

    let id = it.parse(&parse_id, "from import")?;
    let import = it.parse(&parse_import, "from import")?;

    let (en_line, en_pos) = (import.en_line, import.en_pos);
    let node = ASTNode::FromImport { id, import };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Import, "import")?;

    let mut import = vec![];
    it.peek_while_not_tokens(&[Token::As, Token::NL], &mut |it, _, no| {
        import.push(*it.parse(&parse_id, format!("import id {}", no).as_str())?);
        it.eat_if(Token::Comma);
        Ok(())
    })?;
    let _as = it.parse_vec_if(Token::As, &parse_as, "import")?;

    let (en_line, en_pos) = match (import.last(), _as.last()) {
        (_, Some(token_pos)) => (token_pos.en_line, token_pos.en_pos),
        (Some(token_pos), _) => (token_pos.en_line, token_pos.en_pos),
        (..) => (st_line, st_pos + Token::Import.width())
    };
    let node = ASTNode::Import { import, _as };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_as(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut aliases = vec![];
    it.peek_while_not_token(Token::NL, &mut |it, token_pos, no| match token_pos.token {
        Token::Id(_) => {
            aliases.push(*it.parse(&parse_id, format!("import {}", no).as_str())?);
            it.eat_if(Token::Comma);
            Ok(())
        }
        _ => Err(TokenErr {
            expected: Token::Id(String::new()),
            actual:   token_pos.clone(),
            message:  format!("import {}", no)
        })
    })?;

    Ok(aliases)
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
    let mut doc = None;
    let mut imports = Vec::new();
    let mut modules = Vec::new();
    let mut type_defs = Vec::new();

    let pure = it.eat_if(Token::Pure).is_some();

    it.peek_while_fn(&|_| true, &mut |it, token_pos, _| match &token_pos.token {
        Token::NL => {
            it.eat(Token::NL, "file")?;
            Ok(())
        }
        Token::Import => {
            imports.push(*it.parse(&parse_import, "file")?);
            Ok(())
        }
        Token::From => {
            imports.push(*it.parse(&parse_from_import, "file")?);
            Ok(())
        }
        Token::Type => {
            type_defs.push(*it.parse(&parse_type_def, "file")?);
            Ok(())
        }
        Token::Comment(comment) => {
            let (st_line, st_pos) = it.start_pos()?;
            let (en_line, en_pos) = it.eat(Token::Comment(comment.clone()), "file")?;
            let node = ASTNode::Comment { comment: comment.clone() };
            modules.push(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
            Ok(())
        }
        Token::DocString(comment) =>
            if doc == None {
                doc = Some(comment.clone());
                Ok(())
            } else {
                Err(CustomErr {
                    expected: String::from("Class can only have one docstring"),
                    actual:   token_pos.clone()
                })
            },
        _ => {
            modules.push(*it.parse(&parse_module, "file")?);
            Ok(())
        }
    })?;

    let node = ASTNode::File { doc, pure, imports, modules, type_defs };
    Ok(Box::from(ASTNodePos { st_line: 0, st_pos: 0, en_line: 0, en_pos: 0, node }))
}

pub fn parse_type_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    it.eat(Token::Type, "type definition")?;
    let _type = it.parse(&parse_type, "type definition")?;

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::IsA => {
                it.eat(Token::IsA, "type definition")?;
                let _type = it.parse(&parse_type, "type definition")?;
                let conditions =
                    it.parse_vec_if(Token::When, &parse_conditions, "type definition")?;
                let (en_line, en_pos) = if let Some(token_pos) = conditions.last() {
                    (token_pos.en_line, token_pos.en_pos)
                } else {
                    (_type.en_line, _type.en_pos)
                };

                let node = ASTNode::TypeAlias { _type: _type.clone(), conditions };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => {
                it.eat_if(Token::NL);
                // TODO add parsing of docs
                let doc = None;
                let body = it.parse(&parse_block, "type body")?;
                let (en_line, en_pos) = (body.en_line, body.en_pos);
                let node = ASTNode::TypeDef { doc, _type: _type.clone(), body: Some(body) };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
        },
        {
            // TODO add parsing of docs
            let doc = None;
            let node = ASTNode::TypeDef { _type: _type.clone(), doc, body: None };
            let (en_line, en_pos) = (_type.en_line, _type.en_pos);
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        },
        "type definition"
    )
}
