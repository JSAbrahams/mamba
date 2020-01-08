use crate::lexer::token::Token;
use crate::parser::_type::parse_expression_type;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::expression::parse_inner_expression;
use crate::parser::iterator::LexIterator;
use crate::parser::operation::parse_expression;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_reassignment(pre: &AST, it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("reassignment")?;
    it.eat(&Token::Assign, "reassignment")?;
    let right = it.parse(&parse_expression, "reassignment", &start)?;

    let node = Node::Reassign { left: Box::new(pre.clone()), right: right.clone() };
    Ok(Box::from(AST::new(&start.union(&right.pos), node)))
}

pub fn parse_anon_fun(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("anonymous function")?;
    it.eat(&Token::BSlash, "anonymous function")?;

    let mut args: Vec<AST> = vec![];
    it.peek_while_not_token(&Token::BTo, &mut |it, _| {
        args.push(*it.parse(&parse_expression_type, "anonymous function", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    it.eat(&Token::BTo, "anonymous function")?;

    let body = it.parse(&parse_expression, "anonymous function", &start)?;
    let node = Node::AnonFun { args, body: body.clone() };
    Ok(Box::from(AST::new(&start.union(&body.pos), node)))
}

// TODO re-add postfix function calling
pub fn parse_call(pre: &AST, it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, ast| match ast.token {
            Token::Point => {
                it.eat(&Token::Point, "call")?;
                let property = it.parse(&parse_inner_expression, "call", &pre.pos)?;
                let node = Node::PropertyCall {
                    instance: Box::from(pre.clone()),
                    property: property.clone()
                };
                Ok(Box::from(AST::new(&pre.pos.union(&property.pos), node)))
            }
            Token::LRBrack => {
                it.eat(&Token::LRBrack, "direct call")?;
                let args = it.parse_vec(&parse_arguments, "direct call", &pre.pos)?;
                let end = it.eat(&Token::RRBrack, "direct call")?;
                let node = Node::FunctionCall { name: Box::from(pre.clone()), args };
                Ok(Box::from(AST::new(&pre.pos.union(&end), node)))
            }
            _ => Err(expected_one_of(&[Token::Point, Token::LRBrack], ast, "function call"))
        },
        &[Token::Point, Token::LRBrack],
        "function call"
    )
}

fn parse_arguments(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("arguments")?;
    let mut arguments = vec![];
    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        arguments.push(*it.parse(&parse_expression, "arguments", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    Ok(arguments)
}
