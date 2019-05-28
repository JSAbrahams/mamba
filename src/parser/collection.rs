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
    let (en_line, en_pos) = it.end_pos()?;
    it.eat(Token::RRBrack, "tuple")?;

    Ok(Box::from(if elements.len() == 1 {
        elements[0].clone()
    } else {
        ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } }
    }))
}

fn parse_set(_: &mut TPIterator) -> ParseResult { unimplemented!() }

fn parse_list(_: &mut TPIterator) -> ParseResult { unimplemented!() }

pub fn parse_expressions(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut expressions = vec![];
    it.peek_while_fn(&is_start_expression, &mut |it, _, no| {
        expressions.push(*it.parse(&parse_expression, format!("expression {}", no).as_str())?);
        it.eat_if(Token::Comma);
        Ok(())
    })?;

    Ok(expressions)
}
