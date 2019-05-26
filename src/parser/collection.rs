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
        CustomEOFErr { expected: "collection".to_string() }
    )
}

pub fn parse_tuple(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat_token(Token::LRBrack)?;

    let elements = it.parse_vec(&parse_zero_or_more_expr, "tuple")?;
    let (en_line, en_pos) = it.end_pos()?;
    it.eat_token(Token::RRBrack)?;

    Ok(Box::from(if elements.is_empty() || elements.len() >= 2 {
        ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Tuple { elements } }
    } else {
        elements[0].clone()
    }))
}

fn parse_set(_it: &mut TPIterator) -> ParseResult {
    unimplemented!()
    //    let (st_line, st_pos) = it.start_pos()?;
    //    it.eat(Token::LCBrack);
    //    if let Some(TokenPos { token: Token::RCBrack, .. }) = it.peek() {
    //        let (en_line, en_pos) = it.start_pos()?;
    //        it.next();
    //
    //        let node = ASTNode::Set { elements: vec![] };
    //        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
    //    }
    //
    //    let head = it.parse(parse_expression, "set");
    //
    //    match it.peek() {
    //        Some(TokenPos { token: Token::Ver, .. }) => {
    //            it.eat(Token::Ver)?;
    //            let conditions = it.parse(&parse_zero_or_more_expr,
    // "conditions")?;            let (en_line, en_pos) = it.end_pos()?;
    //
    //            let node = ASTNode::SetBuilder { items: Box::from(head),
    // conditions };            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos,
    // node })        }
    //        _ => {
    //            if let Some(&t) = it.peek() {
    //                if t.token == Token::Comma {
    //                    it.next();
    //                }
    //            }
    //
    //            let mut elements = vec![head];
    //            let tail: Vec<ASTNodePos> = get_zero_or_more!(it, "set");
    //            elements.extend(tail);
    //
    //            let (en_line, en_pos) = it.end_pos()?;
    //            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node:
    // ASTNode::Set { elements } })        }
    //    };
    //
    //    it.eat(Token::RCBrack);
}

fn parse_list(_it: &mut TPIterator) -> ParseResult {
    unimplemented!()
    //    let (st_line, st_pos) = it.start_pos()?;
    //    it.eat(Token::LSBrack);
    //    if let Some(TokenPos { token: Token::RSBrack, .. }) = it.peek() {
    //        let (en_line, en_pos) = it.start_pos()?;
    //        it.next();
    //
    //        let node = ASTNode::List { elements: vec![] };
    //        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
    //    }
    //
    //    let head = it.parse(parse_expression, "list");
    //
    //    if let Some(TokenPos { token: Token::Ver, .. }) = it.peek() {
    //        it.next();
    //        let conditions: Vec<ASTNodePos> = get_zero_or_more!(it, "list
    // builder");        let (en_line, en_pos) = it.end_pos()?;
    //        it.eat(Token::RSBrack);
    //
    //        let node = ASTNode::ListBuilder { items: Box::from(head), conditions
    // };        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
    //    }
    //
    //    if let Some(&t) = it.peek() {
    //        if t.token == Token::Comma {
    //            it.next();
    //        }
    //    }
    //    let mut elements = vec![head];
    //    let tail: Vec<ASTNodePos> = get_zero_or_more!(it, "list");
    //    elements.extend(tail);
    //
    //    let (en_line, en_pos) = it.end_pos()?;
    //    it.eat(Token::RSBrack);
    //
    //    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::List {
    // elements } })
}

pub fn parse_zero_or_more_expr(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    if it.peak_if_fn(&|token_pos| is_start_expression(token_pos)) {
        parse_one_or_more_expr(it)
    } else {
        Ok(vec![])
    }
}

pub fn parse_one_or_more_expr(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let expression = it.parse(&parse_expression, "first expression")?;
    let mut expressions = vec![*expression];

    it.while_fn(&is_start_expression, &mut |it, _| {
        expressions.push(*it.parse(&parse_expression, "expression")?);
        it.eat_if_token(Token::Comma);
        Ok(())
    })?;

    Ok(expressions)
}
