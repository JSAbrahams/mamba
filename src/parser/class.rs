use crate::lexer::token::Token;
use crate::parser::_type::parse_generics;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_type;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::block::parse_block;
use crate::parser::definition::{parse_definition, parse_fun_arg};
use crate::parser::iterator::LexIterator;
use crate::parser::operation::parse_expression;
use crate::parser::parse_result::ParseResult;
use crate::parser::parse_result::{expected, expected_one_of};

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
    it.eat_if(&Token::RSBrack);

    let mut args = vec![];
    if it.eat_if(&Token::LRBrack).is_some() {
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
        it.eat(&Token::RRBrack, "parent arguments")?;
    }

    let end = match (generics.last(), args.last()) {
        (_, Some(node_pos)) => node_pos.pos.clone(),
        (Some(node_pos), _) => node_pos.pos.clone(),
        _ => id.pos.clone()
    };
    let node = Node::Parent { id, generics, args };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}
