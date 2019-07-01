use crate::lexer::token::Token;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expression::is_start_expression;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_collection(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::LRBrack => parse_tuple(it),
            Token::LSBrack => parse_list(it),
            Token::LCBrack => parse_set(it),
            _ => Err(expected_one_of(
                &[Token::LRBrack, Token::LSBrack, Token::LCBrack],
                token_pos,
                "collection"
            ))
        },
        &[Token::LRBrack, Token::LSBrack, Token::LCBrack],
        "collection"
    )
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("tuple")?;
    it.eat(&Token::LRBrack, "tuple")?;

    let elements = it.parse_vec(&parse_expressions)?;
    let (en_line, en_pos) = it.eat(&Token::RRBrack, "tuple")?;

    Ok(Box::from(if elements.len() == 1 {
        elements[0].clone()
    } else {
        ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } }
    }))
}

fn parse_set(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("set")?;
    it.eat(&Token::LCBrack, "set")?;

    if let Some((en_line, en_pos)) = it.eat_if(&Token::RCBrack) {
        let node = ASTNode::Set { elements: vec![] };
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let item = it.parse(&parse_expression)?;
    if it.eat_if(&Token::Ver).is_some() {
        let conditions = it.parse_vec(&parse_expressions)?;
        let (en_line, en_pos) = it.eat(&Token::RCBrack, "set")?;
        let node = ASTNode::SetBuilder { item, conditions };
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let mut elements = vec![*item];
    elements.append(&mut it.parse_vec_if(&Token::Comma, &parse_expressions, "set")?);

    let (en_line, en_pos) = it.eat(&Token::RCBrack, "set")?;
    let node = ASTNode::Set { elements };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_list(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("list")?;
    it.eat(&Token::LSBrack, "list")?;

    if let Some((en_line, en_pos)) = it.eat_if(&Token::RSBrack) {
        let node = ASTNode::List { elements: vec![] };
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let item = it.parse(&parse_expression)?;
    if it.eat_if(&Token::Ver).is_some() {
        let conditions = it.parse_vec(&parse_expressions)?;
        let (en_line, en_pos) = it.eat(&Token::RSBrack, "list")?;
        let node = ASTNode::ListBuilder { item, conditions };
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let mut elements = vec![*item];
    elements.append(&mut it.parse_vec_if(&Token::Comma, &parse_expressions, "list")?);

    let (en_line, en_pos) = it.eat(&Token::RSBrack, "list")?;
    let node = ASTNode::List { elements };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_expressions(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut expressions = vec![];
    it.peek_while_fn(&is_start_expression, &mut |it, _| {
        expressions.push(*it.parse(&parse_expression)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    Ok(expressions)
}
