use std::ops::Deref;

use crate::lexer::token::Token;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::LexIterator;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_id(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match &lex.token {
            Token::_Self => {
                let end = it.eat(&Token::_Self, "identifier")?;
                Ok(Box::from(AST::new(&end, Node::_Self)))
            }
            Token::Init => {
                let end = it.eat(&Token::Init, "identifier")?;
                Ok(Box::from(AST::new(&end, Node::Init)))
            }
            Token::Id(id) => {
                let end = it.eat(&Token::Id(id.clone()), "identifier")?;
                Ok(Box::from(AST::new(&end, Node::Id { lit: id.clone() })))
            }
            Token::LRBrack => {
                let mut elements = vec![];
                let start = it.eat(&Token::LRBrack, "identifier tuple")?;
                // TODO allow id's to be mutable within tuples
                it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
                    elements.push(*it.parse(&parse_id, "identifier", &start)?);
                    it.eat_if(&Token::Comma);
                    Ok(())
                })?;

                let end = it.eat(&Token::RRBrack, "identifier tuple")?;
                Ok(Box::from(AST::new(&end, Node::Tuple { elements })))
            }
            _ => Err(expected_one_of(
                &[Token::_Self, Token::Init, Token::Id(String::new())],
                lex,
                "identifier"
            ))
        },
        &[Token::_Self, Token::Init, Token::Id(String::new())],
        "identifier"
    )
}

pub fn parse_generics(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("generics")?;
    let mut generics: Vec<AST> = Vec::new();

    it.peek_while_not_token(&Token::RSBrack, &mut |it, _| {
        generics.push(*it.parse(&parse_generic, "generics", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    Ok(generics)
}

fn parse_generic(it: &mut LexIterator) -> ParseResult {
    let start = &it.start_pos("generic")?;
    let id = it.parse(&parse_id, "generic", start)?;
    let isa = it.parse_if(&Token::IsA, &parse_id, "generic", start)?;
    let end = isa.clone().map_or(id.pos.clone(), |isa| isa.pos);

    let node = Node::Generic { id, isa };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_type(it: &mut LexIterator) -> ParseResult {
    let start = &it.start_pos("type")?;
    let _type = it.peek_or_err(
        &|it, lex| match lex.token {
            Token::Id(_) => {
                let id = it.parse(&parse_id, "type", start)?;
                let generics =
                    it.parse_vec_if(&Token::LSBrack, &parse_generics, "type generic", start)?;
                let end = match (it.eat_if(&Token::RSBrack), generics.last()) {
                    (Some(end), _) => end.clone(),
                    (_, Some(generic)) => generic.pos.clone(),
                    _ => id.pos.clone()
                };

                let node = Node::Type { id, generics };
                Ok(Box::from(AST::new(&start.union(&end), node)))
            }
            Token::LRBrack => it.parse(&parse_type_tuple, "type", start),
            Token::LCBrack => it.parse(&parse_type_set, "type", start),
            _ => Err(expected_one_of(
                &[Token::Id(String::new()), Token::LRBrack, Token::LCBrack],
                &lex.clone(),
                "type"
            ))
        },
        &[Token::Id(String::new()), Token::LRBrack],
        "type"
    )?;

    let _type = if it.peak_if_fn(&|lex| lex.token == Token::Question) {
        it.eat(&Token::Question, "optional type");
        Box::from(AST { pos: _type.pos.clone(), node: Node::QuestionOp { expr: _type.clone() } })
    } else {
        _type
    };

    let res = it.parse_if(
        &Token::To,
        &|it| {
            let ret_ty = it.parse(&parse_type, "type", start)?;
            let args = match &_type.node {
                Node::TypeTup { types } => types.clone(),
                _ => vec![_type.deref().clone()]
            };

            let node = Node::TypeFun { args, ret_ty: ret_ty.clone() };
            Ok(Box::from(AST::new(&start.union(&ret_ty.pos), node)))
        },
        "function type",
        start
    )?;

    match res {
        Some(ast_node_pos) => Ok(ast_node_pos),
        None => Ok(_type.clone())
    }
}

pub fn parse_conditions(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
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

fn parse_condition(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("condition")?;
    let cond = it.parse(&parse_expression, "condition", &start)?;
    let _else = it.parse_if(&Token::Else, &parse_expression, "condition else", &start)?;
    let end = _else.clone().map_or(cond.pos.clone(), |e| e.pos);

    let node = Node::Condition { cond, _else };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_type_set(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type set")?;
    it.eat(&Token::LCBrack, "type set")?;

    let mut types = vec![];
    it.peek_while_not_token(&Token::RCBrack, &mut |it, _| {
        types.push(*it.parse(&parse_type, "type set", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    let end = it.eat(&Token::RCBrack, "type set")?;
    let node = Node::TypeUnion { types };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_type_tuple(it: &mut LexIterator) -> ParseResult {
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
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_id_maybe_type(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("identifier maybe type")?;
    let mutable = it.eat_if(&Token::Mut).is_some();
    let id = it.parse(&parse_id, "identifier maybe type", &start)?;
    let _type = it.parse_if(&Token::DoublePoint, &parse_type, "identifier maybe type", &start)?;
    let end = _type.clone().map_or(id.pos.clone(), |t| t.pos);

    let node = Node::IdType { id, mutable, _type };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}
