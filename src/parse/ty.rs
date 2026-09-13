use std::ops::Deref;

use crate::common::position::Position;
use crate::parse::ast::{Node, AST};
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::result::{custom, expected_one_of, ParseResult};

/// Parse an identifier, or a tuple of identifiers.
///
/// At a binding site each tuple element may carry its own [Token::Mut], which is what
/// `binding` allows. Elsewhere, such as a `for` loop's variable, nothing is bound and `mut`
/// is meaningless, so it is not accepted.
fn parse_id_or_tuple(it: &mut LexIterator, binding: bool) -> ParseResult {
    let expected = [Token::Id(String::new()), Token::LRBrack];

    it.peek_or_err(
        &|it, lex| match &lex.token {
            Token::Id(id) => {
                let end = it.eat(&Token::Id(id.clone()), "identifier")?;
                Ok(Box::from(AST::new(end, Node::Id { lit: id.clone() })))
            }
            Token::LRBrack => {
                let start = it.eat(&Token::LRBrack, "identifier tuple")?;
                let elements = it.parse_comma_separated(
                    &Token::RRBrack,
                    &|it| parse_tuple_element(it, binding),
                    "identifier tuple",
                    start,
                )?;

                let end = it.eat(&Token::RRBrack, "identifier tuple")?;
                let elements = elements.into_iter().map(|e| *e).collect();
                Ok(Box::from(AST::new(end, Node::Tuple { elements })))
            }
            _ => Err(Box::from(expected_one_of(&expected, lex, "identifier"))),
        },
        &expected,
        "identifier",
    )
}

/// Eat an optional [Token::Mut], then parse the identifier it marks.
///
/// A tuple is one binding per element, so the marker goes on the elements and never on the
/// tuple itself. This holds at every depth, so `(mut (a, b), c)` is rejected just as
/// `mut (a, b)` is, rather than silently ignoring the marker.
fn parse_maybe_mut_id(
    it: &mut LexIterator,
    binding: bool,
    start: Position,
) -> ParseResult<(bool, Box<AST>)> {
    let mutable = binding && it.eat_if(&Token::Mut).is_some();
    let expr = it.parse(&|it| parse_id_or_tuple(it, binding), "identifier", start)?;

    if mutable && matches!(expr.node, Node::Tuple { .. }) {
        let msg = format!("Cannot mark an identifier tuple '{}'", Token::Mut);
        return Err(Box::from(custom(&msg, start.union(expr.pos))));
    }
    Ok((mutable, expr))
}

/// One element of an identifier tuple, which never takes a type annotation.
///
/// At a binding site the element carries its own [Token::Mut], and is wrapped in a
/// [Node::ExpressionType] to hold it. Elsewhere the identifier is returned bare.
fn parse_tuple_element(it: &mut LexIterator, binding: bool) -> ParseResult {
    let start = it.start_pos("identifier tuple element")?;
    let (mutable, expr) = parse_maybe_mut_id(it, binding, start)?;

    if let Some(annotation_pos) = it.eat_if(&Token::DoublePoint) {
        return Err(Box::from(custom(
            "Type annotation not allowed here",
            annotation_pos,
        )));
    }
    if !binding {
        return Ok(expr);
    }

    let end = expr.pos;
    let node = Node::ExpressionType {
        expr,
        mutable,
        ty: None,
    };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_id(it: &mut LexIterator) -> ParseResult {
    parse_id_or_tuple(it, false)
}

pub fn parse_generics(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("generics")?;
    let generics = it.parse_comma_separated(&Token::RSBrack, &parse_generic, "generics", start)?;
    Ok(generics.into_iter().map(|g| *g).collect())
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
    let expected = [Token::Id(String::new()), Token::LRBrack, Token::LCBrack];

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
            _ => Err(Box::from(expected_one_of(&expected, &lex.clone(), "type"))),
        },
        &expected,
        "type",
    )?;

    let ty = if it.peek_if(&|lex| lex.token == Token::Question) {
        it.eat(&Token::Question, "optional type")?;
        Box::from(AST {
            pos: ty.pos,
            node: Node::QuestionOp { expr: ty },
        })
    } else {
        ty
    };

    let res = it.parse_if(
        &Token::To,
        &|it| {
            let ret_ty = it.parse(&parse_type, "type", start)?;
            let args = match &ty.node {
                Node::TypeTup { types } => types.clone(),
                _ => vec![ty.deref().clone()],
            };

            let node = Node::TypeFun {
                args,
                ret_ty: ret_ty.clone(),
            };
            Ok(Box::from(AST::new(start.union(ret_ty.pos), node)))
        },
        "function type",
        start,
    )?;

    match res {
        Some(ast) => Ok(ast),
        None => Ok(ty),
    }
}

pub fn parse_type_set(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type set")?;
    it.eat(&Token::LCBrack, "type set")?;

    let types = it.parse_comma_separated(&Token::RCBrack, &parse_type, "type set", start)?;
    let end = it.eat(&Token::RCBrack, "type set")?;

    let types = types.into_iter().map(|t| *t).collect();
    let node = Node::TypeUnion { types };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_type_tuple(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type tuple")?;
    it.eat(&Token::LRBrack, "type tuple")?;

    let types = it.parse_comma_separated(&Token::RRBrack, &parse_type, "type tuple", start)?;
    let end = it.eat(&Token::RRBrack, "type tuple")?;

    let types = types.into_iter().map(|t| *t).collect();
    let node = Node::TypeTup { types };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_expression_type(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("expression type")?;
    let (mutable, expr) = parse_maybe_mut_id(it, true, start)?;
    let ty = it.parse_if(&Token::DoublePoint, &parse_type, "expression type", start)?;
    let end = ty.clone().map_or(expr.pos, |t| t.pos);

    let node = Node::ExpressionType { expr, mutable, ty };
    Ok(Box::from(AST::new(start.union(end), node)))
}
