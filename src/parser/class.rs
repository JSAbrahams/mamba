use crate::lexer::token::Token;
use crate::parser::_type::parse_generics;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::definition::parse_fun_arg;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected;
use crate::parser::parse_result::ParseResult;

pub fn parse_class(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("class")?;
    it.eat(&Token::Class, "class")?;
    let _type = it.parse(&parse_type, "class", st_line, st_pos)?;

    let mut args = vec![];
    if it.eat_if(&Token::LRBrack).is_some() {
        it.peek_while_not_token(&Token::RRBrack, &mut |it, token_pos| match token_pos.token {
            Token::Def => {
                it.eat(&Token::Def, "constructor argument")?;
                args.push(*it.parse(&parse_fun_arg, "constructor argument", st_line, st_pos)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(expected(&Token::Def, &token_pos.clone(), "class"))
        })?;
        it.eat(&Token::RRBrack, "class arguments")?;
    }

    let mut parents = vec![];
    if it.eat_if(&Token::IsA).is_some() {
        it.peek_while_not_token(&Token::NL, &mut |it, token_pos| match token_pos.token {
            Token::Id(_) => {
                parents.push(*it.parse(&parse_parent, "parents", st_line, st_pos)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(expected(&Token::Id(String::new()), &token_pos.clone(), "parents"))
        })?;
    }

    it.eat(&Token::NL, "class")?;
    let body = it.parse(&parse_block, "class", st_line, st_pos)?;
    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::Class { _type, args, parents, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_parent(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("parent")?;

    let id = it.parse(&parse_id, "parent", st_line, st_pos)?;
    let generics =
        it.parse_vec_if(&Token::LSBrack, &parse_generics, "parent generics", st_line, st_pos)?;
    it.eat_if(&Token::RSBrack);
    let mut args = vec![];
    if it.eat_if(&Token::LRBrack).is_some() {
        it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
            args.push(*it.parse(&parse_expression, "parent", st_line, st_pos)?);
            it.eat_if(&Token::Comma);
            Ok(())
        })?;
        it.eat(&Token::RRBrack, "parent arguments")?;
    }

    let (en_line, en_pos) = match (generics.last(), args.last()) {
        (_, Some(tp)) => (tp.en_line, tp.en_pos),
        (Some(tp), _) => (tp.en_line, tp.en_pos),
        _ => (id.en_line, id.en_pos)
    };
    let node = ASTNode::Parent { id, generics, args };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}
