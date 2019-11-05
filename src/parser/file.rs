use crate::common::position::Position;
use crate::lexer::token::Token;
use crate::parser::_type::parse_conditions;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_type;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::block::parse_block;
use crate::parser::block::parse_statements;
use crate::parser::class::parse_class;
use crate::parser::iterator::LexIterator;
use crate::parser::parse_result::expected;
use crate::parser::parse_result::ParseResult;

pub fn parse_from_import(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("from import")?;
    it.eat(&Token::From, "from import")?;
    let id = it.parse(&parse_id, "from import", &start)?;
    let import = it.parse(&parse_import, "from import", &start)?;

    let node = Node::FromImport { id, import: import.clone() };
    Ok(Box::from(AST::new(&start.union(&import.pos), node)))
}

pub fn parse_import(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("import")?;
    let end = it.eat(&Token::Import, "import")?;
    let mut import = vec![];
    it.peek_while_not_tokens(&[Token::As, Token::NL], &mut |it, _| {
        import.push(*it.parse(&parse_id, "import", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    let _as = it.parse_vec_if(&Token::As, &parse_as, "import", &start)?;

    let end = match (import.last(), _as.last()) {
        (_, Some(ast)) => ast.pos.clone(),
        (Some(ast), _) => ast.pos.clone(),
        (..) => end
    };
    Ok(Box::from(AST::new(&start.union(&end), Node::Import { import, _as })))
}

fn parse_as(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("as")?;
    let mut aliases = vec![];
    it.peek_while_not_token(&Token::NL, &mut |it, lex| match lex.token {
        Token::Id(_) => {
            aliases.push(*it.parse(&parse_id, "as", &start)?);
            it.eat_if(&Token::Comma);
            Ok(())
        }
        _ => Err(expected(&Token::Id(String::new()), lex, "as"))
    })?;

    Ok(aliases)
}

pub fn parse_script(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("script")?;
    let statements = it.parse_vec(&parse_statements, "script", &start)?;
    let end = statements.last().map_or(start.clone(), |last| last.pos.clone());

    let node = Node::Script { statements };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_module(it: &mut LexIterator) -> ParseResult {
    if it.peek_if(&|lex| lex.token == Token::Class) {
        parse_class(it)
    } else {
        parse_script(it)
    }
}

pub fn parse_file(it: &mut LexIterator) -> ParseResult {
    let start = Position::default();
    let mut imports = Vec::new();
    let mut comments = Vec::new();
    let mut modules = Vec::new();

    let pure = it.eat_if(&Token::Pure).is_some();

    it.peek_while_fn(&|_| true, &mut |it, lex| match &lex.token {
        Token::NL => {
            it.eat(&Token::NL, "file")?;
            Ok(())
        }
        Token::Import => {
            imports.push(*it.parse(&parse_import, "file", &start)?);
            Ok(())
        }
        Token::From => {
            imports.push(*it.parse(&parse_from_import, "file", &start)?);
            Ok(())
        }
        Token::Comment(comment) => {
            let start = it.start_pos("comment")?;
            let end = it.eat(&Token::Comment(comment.clone()), "file")?;
            let node = Node::Comment { comment: comment.clone() };
            comments.push(AST::new(&start.union(&end), node));
            Ok(())
        }
        Token::Type => {
            modules.push(*it.parse(&parse_type_def, "file", &start)?);
            Ok(())
        }
        _ => {
            modules.push(*it.parse(&parse_module, "file", &start)?);
            Ok(())
        }
    })?;

    let node = Node::File { pure, comments, imports, modules };
    Ok(Box::from(AST::new(&start, node)))
}

pub fn parse_type_def(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type definition")?;
    it.eat(&Token::Type, "type definition")?;
    let _type = it.parse(&parse_type, "type definition", &start)?;

    it.peek(
        &|it, lex| match lex.token {
            Token::IsA => {
                it.eat(&Token::IsA, "type definition")?;
                let alias = it.parse(&parse_type, "type definition", &start)?;
                let conditions =
                    it.parse_vec_if(&Token::When, &parse_conditions, "type definition", &start)?;
                let end = conditions.last().map_or(_type.pos.clone(), |cond| cond.pos.clone());

                let node = Node::TypeAlias { _type: _type.clone(), alias, conditions };
                Ok(Box::from(AST::new(&start.union(&end), node)))
            }
            _ => {
                it.eat_if(&Token::NL);
                let body = it.parse(&parse_block, "type definition", &start)?;
                let node = Node::TypeDef { _type: _type.clone(), body: Some(body.clone()) };
                Ok(Box::from(AST::new(&start.union(&body.pos), node)))
            }
        },
        {
            let node = Node::TypeDef { _type: _type.clone(), body: None };
            Ok(Box::from(AST::new(&start.union(&_type.pos), node)))
        }
    )
}
