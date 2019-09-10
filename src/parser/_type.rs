use crate::lexer::token::Token;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;
use std::ops::Deref;

pub fn parse_id(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match &token_pos.token {
            Token::_Self => {
                let end = it.eat(&Token::_Self, "identifier")?;
                Ok(Box::from(AST::new(&token_pos.start, &end, Node::_Self)))
            }
            Token::Init => {
                let end = it.eat(&Token::Init, "identifier")?;
                Ok(Box::from(AST::new(&token_pos.start, &end, Node::Init)))
            }
            Token::Id(id) => {
                let end = it.eat(&Token::Id(id.clone()), "identifier")?;
                Ok(Box::from(AST::new(&token_pos.start, &end, Node::Id { lit: id.clone() })))
            }
            _ => Err(expected_one_of(
                &[Token::_Self, Token::Init, Token::Id(String::new())],
                token_pos,
                "identifier"
            ))
        },
        &[Token::_Self, Token::Init, Token::Id(String::new())],
        "identifier"
    )
}

pub fn parse_generics(it: &mut TPIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("generics")?;
    let mut generics: Vec<AST> = Vec::new();

    it.peek_while_not_token(&Token::RSBrack, &mut |it, _| {
        generics.push(*it.parse(&parse_generic, "generics", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    Ok(generics)
}

fn parse_generic(it: &mut TPIterator) -> ParseResult {
    let start = &it.start_pos("generic")?;
    let id = it.parse(&parse_id, "generic", start)?;
    let isa = it.parse_if(&Token::IsA, &parse_id, "generic", start)?;
    let end = isa.clone().map_or(id.pos.end.clone(), |isa| isa.pos.end);

    let node = Node::Generic { id, isa };
    Ok(Box::from(AST::new(start, &end, node)))
}

pub fn parse_type(it: &mut TPIterator) -> ParseResult {
    let start = &it.start_pos("type")?;
    let _type = it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Id(_) => {
                let id = it.parse(&parse_id, "type", start)?;
                let generics =
                    it.parse_vec_if(&Token::LSBrack, &parse_generics, "type generic", start)?;
                let end = match (it.eat_if(&Token::RSBrack), generics.last()) {
                    (Some(end), _) => end.clone(),
                    (_, Some(generic)) => generic.pos.end.clone(),
                    _ => id.pos.end.clone()
                };

                let node = Node::Type { id, generics };
                Ok(Box::from(AST::new(start, &end, node)))
            }
            Token::LRBrack => it.parse(&parse_type_tuple, "type", start),
            _ => Err(expected_one_of(
                &[Token::Id(String::new()), Token::LRBrack],
                &token_pos.clone(),
                "type"
            ))
        },
        &[Token::Id(String::new()), Token::LRBrack],
        "type"
    )?;

    let res = it.parse_if(
        &Token::To,
        &|it| {
            let ret_ty = it.parse(&parse_type, "type", start)?;
            let args = match &_type.node {
                Node::TypeTup { types } => types.clone(),
                _ => vec![_type.deref().clone()]
            };

            let node = Node::TypeFun { args, ret_ty: ret_ty.clone() };
            Ok(Box::from(AST::new(&start, &ret_ty.pos.end, node)))
        },
        "function type",
        start
    )?;

    match res {
        Some(ast_node_pos) => Ok(ast_node_pos),
        None => Ok(_type.clone())
    }
}

pub fn parse_conditions(it: &mut TPIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("conditions")?;
    let mut conditions = vec![];

    if it.eat_if(&Token::NL).is_some() {
        it.eat(&Token::Indent, "conditions")?;
        it.peek_while_not_token(&Token::Dedent, &mut |it, _| {
            conditions.push(*it.parse(&parse_condition, "conditions", &start)?);
            it.eat_if(&Token::NL);
            Ok(())
        })?;
        it.eat(&Token::Dedent, "conditions")?;
    } else {
        let start = it.start_pos("conditions")?;
        conditions.push(*it.parse(&parse_condition, "conditions", &start)?);
    }

    Ok(conditions)
}

fn parse_condition(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("condition")?;
    let cond = it.parse(&parse_expression, "condition", &start)?;
    let _else = it.parse_if(&Token::Else, &parse_expression, "condition else", &start)?;
    let end = _else.clone().map_or(cond.pos.end.clone(), |e| e.pos.end);

    let node = Node::Condition { cond, _else };
    Ok(Box::from(AST::new(&start, &end, node)))
}

pub fn parse_type_tuple(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("type tuple")?;
    it.eat(&Token::LRBrack, "type tuple")?;

    let mut types = vec![];
    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        types.push(*it.parse(&parse_type, "type tuple", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    let end = it.eat(&Token::RRBrack, "type tuple")?;
    let node = Node::TypeTup { types };
    Ok(Box::from(AST::new(&start, &end, node)))
}

pub fn parse_id_maybe_type(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("identifier maybe type")?;
    let mutable = it.eat_if(&Token::Mut).is_some();
    let id = it.parse(&parse_id, "identifier maybe type", &start)?;
    let _type = it.parse_if(&Token::DoublePoint, &parse_type, "identifier maybe type", &start)?;
    let end = _type.clone().map_or(id.pos.end.clone(), |t| t.pos.end);

    let node = Node::IdType { id, mutable, _type };
    Ok(Box::from(AST::new(&start, &end, node)))
}
