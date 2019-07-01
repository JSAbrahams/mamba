use crate::lexer::token::Token;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expression::is_start_expression_exclude_unary;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_reassignment(pre: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("reassignment")?;
    it.eat(&Token::Assign, "reassignment")?;

    let right = it.parse(&parse_expression, "reassignment", st_line, st_pos)?;

    let (en_line, en_pos) = (right.en_line, right.en_pos);
    let node = ASTNode::Reassign { left: Box::new(pre.clone()), right };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_anon_fun(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("anonymous function")?;
    it.eat(&Token::BSlash, "anonymous function")?;

    let mut args: Vec<ASTNodePos> = vec![];
    it.peek_while_not_token(&Token::BTo, &mut |it, _| {
        args.push(*it.parse(&parse_id_maybe_type, "anonymous function", st_line, st_pos)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    it.eat(&Token::BTo, "anonymous function")?;

    let body = it.parse(&parse_expression, "anonymous function", st_line, st_pos)?;
    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::AnonFun { args, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_call(pre: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);

    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Point => {
                it.eat(&Token::Point, "call")?;
                let property = it.parse(&parse_expression, "call", st_line, st_pos)?;
                let (en_line, en_pos) = (property.en_line, property.en_pos);

                let node = ASTNode::PropertyCall { instance: Box::from(pre.clone()), property };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            Token::LRBrack => {
                it.eat(&Token::LRBrack, "direct call")?;
                let args = it.parse_vec(&parse_arguments, "direct call", st_line, st_pos)?;
                let (en_line, en_pos) = it.eat(&Token::RRBrack, "direct call")?;

                let node = ASTNode::FunctionCall { name: Box::from(pre.clone()), args };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ if is_start_expression_exclude_unary(token_pos) => {
                let arg = it.parse(&parse_expression, "call", st_line, st_pos)?;
                let (en_line, en_pos) = (arg.en_line, arg.en_pos);

                let node = ASTNode::FunctionCall { name: Box::from(pre.clone()), args: vec![*arg] };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => Err(expected_one_of(&[Token::Point, Token::LRBrack], token_pos, "function call"))
        },
        &[Token::Point, Token::LRBrack],
        "function call"
    )
}

fn parse_arguments(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let (st_line, st_pos) = it.start_pos("arguments")?;
    let mut arguments = Vec::new();

    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        arguments.push(*it.parse(&parse_expression, "arguments", st_line, st_pos)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    Ok(arguments)
}
