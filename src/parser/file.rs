use crate::lexer::token::Token;
use crate::parser::_type::parse_conditions;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::class::parse_class;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected;
use crate::parser::parse_result::ParseResult;

pub fn parse_from_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("from import")?;
    it.eat(&Token::From, "from import")?;

    let id = it.parse(&parse_id, "from import", st_line, st_pos)?;
    let import = it.parse(&parse_import, "from import", st_line, st_pos)?;

    let (en_line, en_pos) = (import.en_line, import.en_pos);
    let node = ASTNode::FromImport { id, import };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_import(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("import")?;
    it.eat(&Token::Import, "import")?;

    let mut import = vec![];
    it.peek_while_not_tokens(&[Token::As, Token::NL], &mut |it, _| {
        import.push(*it.parse(&parse_id, "import", st_line, st_pos)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    let _as = it.parse_vec_if(&Token::As, &parse_as, "import", st_line, st_pos)?;

    let (en_line, en_pos) = match (import.last(), _as.last()) {
        (_, Some(token_pos)) => (token_pos.en_line, token_pos.en_pos),
        (Some(token_pos), _) => (token_pos.en_line, token_pos.en_pos),
        (..) => (st_line, st_pos + Token::Import.width())
    };
    let node = ASTNode::Import { import, _as };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_as(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let (st_line, st_pos) = it.start_pos("as")?;
    let mut aliases = vec![];

    it.peek_while_not_token(&Token::NL, &mut |it, token_pos| match token_pos.token {
        Token::Id(_) => {
            aliases.push(*it.parse(&parse_id, "as", st_line, st_pos)?);
            it.eat_if(&Token::Comma);
            Ok(())
        }
        _ => Err(expected(&Token::Id(String::new()), token_pos, "as"))
    })?;

    Ok(aliases)
}

pub fn parse_file(it: &mut TPIterator) -> ParseResult {
    let mut imports = Vec::new();
    let mut classes = Vec::new();
    let mut type_defs = Vec::new();
    let mut statements = Vec::new();

    let pure = it.eat_if(&Token::Pure).is_some();

    it.peek_while_fn(&|_| true, &mut |it, token_pos| {
        match &token_pos.token {
            Token::NL => {
                it.eat(&Token::NL, "file")?;
            }
            Token::Import => imports.push(*it.parse(&parse_import, "file", 1, 1)?),
            Token::From => imports.push(*it.parse(&parse_from_import, "file", 1, 1)?),
            Token::Type => type_defs.push(*it.parse(&parse_type_def, "file", 1, 1)?),
            Token::Comment(comment) => {
                let (st_line, st_pos) = it.start_pos("comment")?;
                let (en_line, en_pos) = it.eat(&Token::Comment(comment.clone()), "file")?;
                let node = ASTNode::Comment { comment: comment.clone() };
                statements.push(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
            }
            Token::Class => classes.push(*it.parse(&parse_class, "file", 1, 1)?),
            _ => {
                statements.push(*it.parse(&parse_expr_or_stmt, "file", 1, 1)?);
                if it.peak_if_fn(&|token_pos| token_pos.token != Token::NL) {
                    return Err(expected(&Token::NL, &token_pos.clone(), "file"));
                }
            }
        }
        Ok(())
    })?;

    let node = ASTNode::File { pure, imports, classes, type_defs, statements };
    Ok(Box::from(ASTNodePos { st_line: 1, st_pos: 1, en_line: 1, en_pos: 1, node }))
}

pub fn parse_type_def(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("type definition")?;
    it.eat(&Token::Type, "type definition")?;
    let _type = it.parse(&parse_type, "type definition", st_line, st_pos)?;

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::IsA => {
                it.eat(&Token::IsA, "type definition")?;
                let _type = it.parse(&parse_type, "type definition", st_line, st_pos)?;
                let conditions = it.parse_vec_if(
                    &Token::When,
                    &parse_conditions,
                    "type definition",
                    st_line,
                    st_pos
                )?;
                let (en_line, en_pos) = if let Some(token_pos) = conditions.last() {
                    (token_pos.en_line, token_pos.en_pos)
                } else {
                    (_type.en_line, _type.en_pos)
                };

                let node = ASTNode::TypeAlias { _type: _type.clone(), conditions };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => {
                it.eat_if(&Token::NL);
                let body = it.parse(&parse_block, "type definition", st_line, st_pos)?;
                let (en_line, en_pos) = (body.en_line, body.en_pos);
                let node = ASTNode::TypeDef { _type: _type.clone(), body: Some(body) };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
        },
        {
            let node = ASTNode::TypeDef { _type: _type.clone(), body: None };
            let (en_line, en_pos) = (_type.en_line, _type.en_pos);
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        }
    )
}
