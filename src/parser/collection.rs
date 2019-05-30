use crate::lexer::token::Token;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expression::is_start_expression;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_collection(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::LRBrack => parse_tuple(it),
            Token::LSBrack => parse_list(it),
            Token::LCBrack => parse_set(it),
            _ => Err(CustomErr { expected: "collection".to_string(), actual: token_pos.clone() })
        },
        "collection"
    )
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::LRBrack, "tuple")?;

    let elements = it.parse_vec(&parse_expressions, "tuple")?;
    let (en_line, en_pos) = it.eat(Token::RRBrack, "tuple")?;

    Ok(Box::from(if elements.len() == 1 {
        elements[0].clone()
    } else {
        ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } }
    }))
}

fn parse_set(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::LCBrack, "set")?;

    if let Some((en_line, en_pos)) = it.eat_if(Token::RCBrack) {
        let node = ASTNode::Set { elements: vec![] };
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let item = it.parse(&parse_expression, "set")?;
    if it.eat_if(Token::Ver).is_some() {
        let conditions = it.parse_vec(&parse_expressions, "set conditions")?;
        let (en_line, en_pos) = it.eat(Token::RCBrack, "set")?;
        let node = ASTNode::SetBuilder { item, conditions };
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let mut elements = vec![*item];
    elements.append(&mut it.parse_vec_if(Token::Comma, &parse_expressions, "set")?);

    let (en_line, en_pos) = it.eat(Token::RCBrack, "set")?;
    let node = ASTNode::Set { elements };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_list(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::LSBrack, "list")?;

    if let Some((en_line, en_pos)) = it.eat_if(Token::RSBrack) {
        let node = ASTNode::List { elements: vec![] };
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let item = it.parse(&parse_expression, "list")?;
    if it.eat_if(Token::Ver).is_some() {
        let conditions = it.parse_vec(&parse_expressions, "list conditions")?;
        let (en_line, en_pos) = it.eat(Token::RSBrack, "list")?;
        let node = ASTNode::ListBuilder { item, conditions };
        return Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }));
    }

    let mut elements = vec![*item];
    elements.append(&mut it.parse_vec_if(Token::Comma, &parse_expressions, "list")?);

    let (en_line, en_pos) = it.eat(Token::RSBrack, "list")?;
    let node = ASTNode::List { elements };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_expressions(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut expressions = vec![];
    it.peek_while_fn(&is_start_expression, &mut |it, _, no| {
        expressions.push(*it.parse(&parse_expression, format!("expression {}", no).as_str())?);
        it.eat_if(Token::Comma);
        Ok(())
    })?;

    Ok(expressions)
}
