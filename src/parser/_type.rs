use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_id(it: &mut TPIterator) -> ParseResult {
    it.next(
        &|token| match token {
            TokenPos { token: Token::_Self, st_line, st_pos } => {
                let (st_line, st_pos) = (*st_line, *st_pos);
                let (en_line, en_pos) = it.end_pos()?;
                let node = ASTNode::_Self;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            TokenPos { token: Token::Init, st_line, st_pos } => {
                let (st_line, st_pos) = (*st_line, *st_pos);
                let (en_line, en_pos) = it.end_pos()?;
                let node = ASTNode::Init;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            TokenPos { token: Token::Id(id), st_line, st_pos } => {
                let (st_line, st_pos) = (*st_line, *st_pos);
                let (en_line, en_pos) = it.end_pos()?;
                let node = ASTNode::Id { lit: id.clone() };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            next => Err(CustomErr { expected: String::from("id"), actual: next.clone() })
        },
        EOFErr { expected: Token::Id(String::new()) }
    )
}

pub fn parse_generics(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(Token::LSBrack);

    let mut generics: Vec<ASTNodePos> = Vec::new();
    it.while_some_and_not(Token::RSBrack, &|token_pos| {
        generics.push(*it.parse(&parse_generic, "generic")?);
        it.eat_if(Token::Comma);
        Ok(())
    });

    it.eat(Token::RSBrack);
    Ok(generics)
}

fn parse_generic(it: &mut TPIterator) -> ParseResult {
    let id = it.parse(&parse_id, "generic id")?;
    let isa = it.parse_if(Token::IsA, &parse_id, "generic isa")?;

    let (st_line, st_pos) = it.start_pos()?;
    let (en_line, en_pos) = match isa.as_ref() {
        Some(ast) => (ast.en_line, ast.en_pos),
        None => (id.en_line, id.en_pos)
    };

    let node = ASTNode::Generic { id, isa };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_type(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    let _type = it.peek(
        &|token_pos| match token_pos {
            TokenPos { token: Token::Id(_), .. } => {
                let id = it.parse(&parse_id, "type")?;
                let generics = it.parse_vec_if(Token::LSBrack, &parse_generics, "type generic")?;

                let (en_line, en_pos) = match generics.last() {
                    Some(generic) => (generic.en_line, generic.en_pos),
                    None => (id.en_line, id.en_pos)
                };

                let node = ASTNode::Type { id, generics };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            TokenPos { token: Token::LRBrack, .. } => it.parse(&parse_type_tuple, "type"),
            other => Err(TokenErr { expected: Token::LRBrack, actual: other.clone() })
        },
        EOFErr { expected: Token::LRBrack }
    )?;

    it.peek_or_none(&|token_pos| match token_pos {
        Some(TokenPos { token: Token::To, .. }) => {
            it.eat(Token::To);
            let body = it.parse(&parse_type, "type")?;
            let node = ASTNode::TypeFun { _type, body };
            let (en_line, en_pos) = (body.en_line, body.en_pos);
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        }
        _ => Ok(_type)
    })
}

pub fn parse_conditions(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(Token::When);
    it.eat(Token::NL);
    it.eat(Token::Indent);

    let mut conditions = Vec::new();
    it.while_some_and_not(Token::Dedent, &|token_pos| {
        conditions.push(*it.parse(&parse_condition, "condition")?);
        it.eat_if(Token::NL);
        Ok(())
    })?;

    it.eat_if(Token::Dedent);
    Ok(conditions)
}

fn parse_condition(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let cond = it.parse(&parse_expression, "condition")?;
    let _else = it.parse_if(Token::Else, &parse_expression, "condition else")?;

    let (en_line, en_pos) = if let Some(ast_pos) = _else {
        (ast_pos.en_line, ast_pos.en_pos)
    } else {
        (cond.en_line, cond.en_pos)
    };

    let node = ASTNode::Condition { cond, _else };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_type_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::LRBrack);

    let mut types: Vec<ASTNodePos> = Vec::new();
    it.while_some_and_not(Token::RRBrack, &|token_pos| {
        types.push(*it.parse(&parse_type, "type")?);
        Ok(())
    })?;

    let (en_line, en_pos) = it.end_pos()?;
    it.eat(Token::RRBrack);
    let node = ASTNode::TypeTup { types };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_id_maybe_type(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let mutable = it.eat_if(Token::Mut);
    let id = it.parse(&parse_id, "id maybe type")?;
    let _type = it.parse_if(Token::DoublePoint, &parse_type, "id type")?;
    let (en_line, en_pos) = it.end_pos()?;

    let node = ASTNode::IdType { id, mutable, _type };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}
