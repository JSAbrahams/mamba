use std::ops::Deref;

use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::{custom, expected_one_of};
use crate::parse::result::ParseResult;

pub fn parse_id(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match &lex.token {
            Token::_Self => {
                let end = it.eat(&Token::_Self, "identifier")?;
                Ok(Box::from(AST::new(end, Node::new_self())))
            }
            Token::Init => {
                let end = it.eat(&Token::Init, "identifier")?;
                Ok(Box::from(AST::new(end, Node::new_init())))
            }
            Token::Id(id) => {
                let end = it.eat(&Token::Id(id.clone()), "identifier")?;
                Ok(Box::from(AST::new(end, Node::Id { lit: id.clone() })))
            }
            Token::LRBrack => {
                let mut elements = vec![];
                let start = it.eat(&Token::LRBrack, "identifier tuple")?;
                it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
                    elements.push(*it.parse(&parse_expr_no_type, "identifier", start)?);
                    it.eat_if(&Token::Comma);
                    Ok(())
                })?;

                let end = it.eat(&Token::RRBrack, "identifier tuple")?;
                Ok(Box::from(AST::new(end, Node::Tuple { elements })))
            }
            _ => Err(expected_one_of(
                &[Token::_Self, Token::Init, Token::Id(String::new()), Token::LRBrack],
                lex,
                "identifier",
            ))
        },
        &[Token::_Self, Token::Init, Token::Id(String::new())],
        "identifier",
    )
}

pub fn parse_generics(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("generics")?;
    let mut generics: Vec<AST> = Vec::new();

    it.peek_while_not_token(&Token::RSBrack, &mut |it, _| {
        generics.push(*it.parse(&parse_generic, "generics", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    Ok(generics)
}

pub fn parse_generic(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("generic")?;
    let id = it.parse(&parse_id, "generic", start)?;
    let isa = it.parse_if(&Token::DoublePoint, &parse_id, "generic", start)?;
    let end = isa.clone().map_or(id.pos, |isa| isa.pos);

    let node = Node::Generic { id, isa };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_type(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type")?;
    let ty = it.peek_or_err(
        &|it, lex| match lex.token {
            Token::Id(_) => {
                let id = it.parse(&parse_id, "type", start)?;
                let generics =
                    it.parse_vec_if(&Token::LSBrack, &parse_generics, "type generic", start)?;
                let end = if generics.last().is_some() {
                    it.eat(&Token::RSBrack, "type generics")?
                } else {
                    id.pos
                };

                let node = Node::Type { id, generics };
                Ok(Box::from(AST::new(start.union(end), node)))
            }
            Token::LRBrack => it.parse(&parse_type_tuple, "type", start),
            Token::LCBrack => it.parse(&parse_type_set, "type", start),
            _ => Err(expected_one_of(
                &[Token::Id(String::new()), Token::LRBrack, Token::LCBrack],
                &lex.clone(),
                "type",
            ))
        },
        &[Token::Id(String::new()), Token::LRBrack],
        "type",
    )?;

    let ty = if it.peek_if(&|lex| lex.token == Token::Question) {
        it.eat(&Token::Question, "optional type")?;
        Box::from(AST { pos: ty.pos, node: Node::QuestionOp { expr: ty } })
    } else {
        ty
    };

    let res = it.parse_if(
        &Token::To,
        &|it| {
            let ret_ty = it.parse(&parse_type, "type", start)?;
            let args = match &ty.node {
                Node::TypeTup { types } => types.clone(),
                _ => vec![ty.deref().clone()]
            };

            let node = Node::TypeFun { args, ret_ty: ret_ty.clone() };
            Ok(Box::from(AST::new(start.union(ret_ty.pos), node)))
        },
        "function type",
        start,
    )?;

    match res {
        Some(ast) => Ok(ast),
        None => Ok(ty)
    }
}

pub fn parse_conditions(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("conditions")?;
    let mut conditions = vec![];

    if it.eat_if(&Token::NL).is_some() {
        it.eat(&Token::Indent, "conditions")?;
        it.peek_while_not_token(&Token::Dedent, &mut |it, _| {
            conditions.push(*it.parse(&parse_condition, "conditions", start)?);
            it.eat_if(&Token::NL);
            Ok(())
        })?;
        it.eat(&Token::Dedent, "conditions")?;
    } else {
        let start = it.start_pos("conditions")?;
        conditions.push(*it.parse(&parse_condition, "conditions", start)?);
    }

    Ok(conditions)
}

fn parse_condition(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("condition")?;
    let cond = it.parse(&parse_expression, "condition", start)?;
    let el = it.parse_if(&Token::Else, &parse_expression, "condition else", start)?;
    let end = el.clone().map_or(cond.pos, |e| e.pos);

    let node = Node::Condition { cond, el };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_type_set(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type set")?;
    it.eat(&Token::LCBrack, "type set")?;

    let mut types = vec![];
    it.peek_while_not_token(&Token::RCBrack, &mut |it, _| {
        types.push(*it.parse(&parse_type, "type set", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    let end = it.eat(&Token::RCBrack, "type set")?;
    let node = Node::TypeUnion { types };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_type_tuple(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type tuple")?;
    it.eat(&Token::LRBrack, "type tuple")?;

    let mut types = vec![];
    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        types.push(*it.parse(&parse_type, "type tuple", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    let end = it.eat(&Token::RRBrack, "type tuple")?;
    let node = Node::TypeTup { types };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_expr_no_type(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("expression no type")?;
    let expr = it.parse(&parse_id, "expression no type", start)?;
    if let Some(annotation_pos) = it.eat_if(&Token::DoublePoint) {
        Err(custom("Type annotation not allowed here", annotation_pos))
    } else {
        Ok(expr)
    }
}

pub fn parse_expression_type(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("expression type")?;
    let mutable = it.eat_if(&Token::Fin).is_none();

    let expr = it.parse(&parse_id, "expression type", start)?;
    let ty = it.parse_if(&Token::DoublePoint, &parse_type, "expression type", start)?;
    let end = ty.clone().map_or(expr.pos, |t| t.pos);

    let node = Node::ExpressionType { expr, mutable, ty };
    Ok(Box::from(AST::new(start.union(end), node)))
}
