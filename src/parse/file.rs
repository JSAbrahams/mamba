use crate::common::position::Position;
use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::block::parse_block;
use crate::parse::class::{parse_class, parse_parent};
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::result::{custom, expected};
use crate::parse::result::ParseResult;
use crate::parse::ty::parse_conditions;
use crate::parse::ty::parse_id;
use crate::parse::ty::parse_type;

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
    Ok(Box::from(AST::new(&start.union(&end), Node::Import { import, aliases: _as })))
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

pub fn parse_file(it: &mut LexIterator) -> ParseResult {
    let start = Position::default();
    let pure = it.eat_if(&Token::Pure).is_some();

    let mut statements = Vec::new();
    it.peek_while_fn(&|_| true, &mut |it, lex| {
        match &lex.token {
            Token::NL => {}
            Token::Import => { statements.push(*it.parse(&parse_import, "file", &start)?); }
            Token::From => { statements.push(*it.parse(&parse_from_import, "file", &start)?); }
            Token::DocStr(string) => {
                let start = it.start_pos("doc_string")?;
                let end = it.eat(&Token::DocStr(string.clone()), "file")?;
                let node = Node::DocStr { lit: string.clone() };
                statements.push(AST::new(&start.union(&end), node));
            }
            Token::Comment(comment) => {
                let start = it.start_pos("comment")?;
                let end = it.eat(&Token::Comment(comment.clone()), "file")?;
                let node = Node::Comment { comment: comment.clone() };
                statements.push(AST::new(&start.union(&end), node));
            }
            Token::Type => { statements.push(*it.parse(&parse_type_def, "file", &start)?); }
            Token::Class => { statements.push(*it.parse(&parse_class, "file", &start)?); }
            _ => { statements.push(*it.parse(&parse_expr_or_stmt, "file", &start)?); }
        }

        let last_pos = it.last_pos();
        it.eat_if_not_empty(&Token::NL, "file", &last_pos)?;
        Ok(())
    })?;

    let node = Node::File { pure, statements };
    Ok(Box::from(AST::new(&start, node)))
}

pub fn parse_type_def(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type definition")?;
    it.eat(&Token::Type, "type definition")?;
    let ty = it.parse(&parse_type, "type definition", &start)?;
    let isa = it.parse_if(&Token::DoublePoint, &parse_parent, "type parent", &start)?;

    it.peek(
        &|it, lex| match lex.token {
            Token::When => {
                it.eat(&Token::When, "conditional type")?;
                let isa = isa
                    .clone()
                    .ok_or_else(|| custom("conditional type must have parent type", &lex.pos))?;

                let conditions = it.parse_vec(&parse_conditions, "conditional type", &start)?;
                let end = conditions.last().map_or(ty.pos.clone(), |cond| cond.pos.clone());

                let node = Node::TypeAlias { ty: ty.clone(), isa, conditions };
                Ok(Box::from(AST::new(&start.union(&end), node)))
            }
            _ => {
                // TODO fix such that we can have empty interfaces
                it.eat_if(&Token::NL);
                let body = it.parse(&parse_block, "type definition", &start)?;
                let isa = isa.clone();
                let node = Node::TypeDef { ty: ty.clone(), isa, body: Some(body.clone()) };
                Ok(Box::from(AST::new(&start.union(&body.pos), node)))
            }
        },
        {
            let isa = isa.clone();
            let node = Node::TypeDef { ty: ty.clone(), isa, body: None };
            Ok(Box::from(AST::new(&start.union(&ty.pos), node)))
        },
    )
}
