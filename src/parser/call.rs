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
    let start = it.start_pos("reassignment")?;
    it.eat(&Token::Assign, "reassignment")?;
    let right = it.parse(&parse_expression, "reassignment", &start)?;

    let node = ASTNode::Reassign { left: Box::new(pre.clone()), right: right.clone() };
    Ok(Box::from(ASTNodePos::new(&start, &right.position.end, node)))
}

pub fn parse_anon_fun(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("anonymous function")?;
    it.eat(&Token::BSlash, "anonymous function")?;

    let mut args: Vec<ASTNodePos> = vec![];
    it.peek_while_not_token(&Token::BTo, &mut |it, token_pos| {
        args.push(*it.parse(&parse_id_maybe_type, "anonymous function", &token_pos.start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    it.eat(&Token::BTo, "anonymous function")?;

    let body = it.parse(&parse_expression, "anonymous function", &start)?;
    let node = ASTNode::AnonFun { args, body: body.clone() };
    Ok(Box::from(ASTNodePos::new(&start, &body.position.end, node)))
}

pub fn parse_call(pre: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Point => {
                it.eat(&Token::Point, "call")?;
                let property = it.parse(&parse_expression, "call", &pre.position.start)?;
                let node = ASTNode::PropertyCall {
                    instance: Box::from(pre.clone()),
                    property: property.clone()
                };
                Ok(Box::from(ASTNodePos::new(&pre.position.start, &property.position.end, node)))
            }
            Token::LRBrack => {
                it.eat(&Token::LRBrack, "direct call")?;
                let args = it.parse_vec(&parse_arguments, "direct call", &pre.position.start)?;
                let end = it.eat(&Token::RRBrack, "direct call")?;
                let node = ASTNode::FunctionCall { name: Box::from(pre.clone()), args };
                Ok(Box::from(ASTNodePos::new(&pre.position.start, &end, node)))
            }
            _ if is_start_expression_exclude_unary(token_pos) => {
                let arg = it.parse(&parse_expression, "call", &pre.position.start)?;
                let node = ASTNode::FunctionCall {
                    name: Box::from(pre.clone()),
                    args: vec![*arg.clone()]
                };
                Ok(Box::from(ASTNodePos::new(&pre.position.start, &arg.position.end, node)))
            }
            _ => Err(expected_one_of(&[Token::Point, Token::LRBrack], token_pos, "function call"))
        },
        &[Token::Point, Token::LRBrack],
        "function call"
    )
}

fn parse_arguments(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut arguments = vec![];
    it.peek_while_not_token(&Token::RRBrack, &mut |it, token_pos| {
        arguments.push(*it.parse(&parse_expression, "arguments", &token_pos.start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    Ok(arguments)
}
