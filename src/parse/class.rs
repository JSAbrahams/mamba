use crate::lex::token::Token;
use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::block::parse_block;
use crate::parse::definition::{parse_definition, parse_fun_arg};
use crate::parse::iterator::LexIterator;
use crate::parse::operation::parse_expression;
use crate::parse::result::ParseResult;
use crate::parse::result::{expected, expected_one_of};
use crate::parse::ty::parse_generics;
use crate::parse::ty::parse_id;
use crate::parse::ty::parse_type;

pub fn parse_class(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("class")?;
    it.eat(&Token::Class, "class")?;
    let ty = it.parse(&parse_type, "class", &start)?;

    let mut args = vec![];
    if it.eat_if(&Token::LRBrack).is_some() {
        it.peek_while_not_token(&Token::RRBrack, &mut |it, lex| match lex.token {
            Token::Def => {
                args.push(*it.parse(&parse_definition, "constructor argument", &start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => {
                args.push(*it.parse(&parse_fun_arg, "constructor argument", &start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
        })?;
        it.eat(&Token::RRBrack, "class arguments")?;
    }

    let mut parents = vec![];
    if it.eat_if(&Token::IsA).is_some() {
        it.peek_while_not_token(&Token::NL, &mut |it, lex| match lex.token {
            Token::Id(_) => {
                parents.push(*it.parse(&parse_parent, "parents", &start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(expected(&Token::Id(String::new()), &lex.clone(), "parents"))
        })?;
    }

    it.eat(&Token::NL, "class")?;
    let (body, pos) = if it.peek_if(&|lex| lex.token == Token::Indent) {
        let body = it.parse(&parse_block, "class", &start)?;
        (Some(body.clone()), start.union(&body.pos))
    } else {
        (None, start)
    };

    let node = Node::Class { ty, args, parents, body };
    Ok(Box::from(AST::new(&pos, node)))
}

pub fn parse_parent(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("parent")?;

    let id = it.parse(&parse_id, "parent", &start)?;
    let generics = it.parse_vec_if(&Token::LSBrack, &parse_generics, "parent generics", &start)?;
    let generics_end =
        if let Some(end) = it.eat_if(&Token::RSBrack) { end } else { id.pos.clone() };
    let ty = Box::from(AST { pos: start.union(&generics_end), node: Node::Type { id, generics } });

    let mut args = vec![];
    let end = if it.eat_if(&Token::LRBrack).is_some() {
        it.peek_while_not_token(&Token::RRBrack, &mut |it, lex| match &lex.token {
            Token::Id { .. } => {
                args.push(*it.parse(&parse_id, "parent arguments", &start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            Token::Str { .. } => {
                args.push(*it.parse(&parse_expression, "parent arguments", &start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(expected_one_of(
                &[
                    Token::Id(String::new()),
                    Token::Str(String::new(), vec![]),
                    Token::Int(String::new()),
                    Token::Real(String::new()),
                    Token::ENum(String::new(), String::new()),
                    Token::Bool(true),
                    Token::Bool(false)
                ],
                lex,
                "parent arguments"
            ))
        })?;
        it.eat(&Token::RRBrack, "parent arguments")?
    } else {
        ty.pos.clone()
    };

    let node = Node::Parent { ty, args };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}
