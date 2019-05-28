use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expression::is_start_expression_exclude_unary;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_reassignment(pre: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Assign, "reassignment")?;

    let right = it.parse(&parse_expression, "reassignment")?;

    let (en_line, en_pos) = (right.en_line, right.en_pos);
    let node = ASTNode::Reassign { left: Box::new(pre.clone()), right };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_anon_fun(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::BSlash, "anonymous function")?;

    let mut args: Vec<ASTNodePos> = vec![];
    it.peek_while_not_token(Token::BTo, &mut |it, _, no| {
        args.push(*it.parse(
            &parse_id_maybe_type,
            format!("anonymous function arg (pos {})", no).as_str()
        )?);

        it.eat_if(Token::Comma);
        Ok(())
    })?;

    it.eat(Token::BTo, "anonymous function")?;

    let body = it.parse(&parse_expression, "anonymous function body")?;
    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::AnonFun { args, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_call(pre: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Point => parse_regular_call(pre, it),
            Token::LRBrack => parse_direct_call(pre, it),
            _ if is_start_expression_exclude_unary(token_pos) =>
                parse_postfix_call(pre.clone(), it),
            _ => Err(CustomErr {
                expected: String::from("function call"),
                actual:   token_pos.clone()
            })
        },
        "function call"
    )
}

fn parse_direct_call(pre: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);
    it.eat(Token::LRBrack, "direct call")?;
    let args = it.parse_vec(&parse_arguments, "arguments")?;
    let (en_line, en_pos) = it.eat(Token::RRBrack, "direct call")?;

    let node = ASTNode::DirectCall { name: Box::from(pre.clone()), args };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_regular_call(pre: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);
    it.eat(Token::Point, "regular call")?;
    let name = it.parse(&parse_id, "call name")?;
    it.peek_or_err(
        &|it, token_pos| {
            let pre = pre.clone();
            let name = name.clone();
            match token_pos {
                TokenPos { token: Token::LRBrack, .. } => {
                    it.eat(Token::LRBrack, "regular call")?;
                    let args = it.parse_vec(&parse_arguments, "arguments")?;
                    let (en_line, en_pos) = it.eat(Token::RRBrack, "regular call")?;
                    let node = ASTNode::MethodCall { instance: Box::from(pre), name, args };
                    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
                }
                _ if is_start_expression_exclude_unary(token_pos) => {
                    let args = vec![*it.parse(&parse_expression, "postfix arg")?];
                    let (en_line, en_pos) = match args.last() {
                        Some(tp) => (tp.en_line, tp.en_pos),
                        _ => unreachable!()
                    };
                    let node = ASTNode::MethodCall { instance: Box::from(pre), name, args };
                    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
                }
                _ => {
                    let (en_line, en_pos) = (name.en_line, name.en_pos);
                    let node = ASTNode::Call { left: Box::from(pre), right: name };
                    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
                }
            }
        },
        "regular call"
    )
}

fn parse_arguments(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut arguments = Vec::new();
    it.peek_while_not_token(Token::RRBrack, &mut |it, _, no| {
        arguments.push(*it.parse(&parse_expression, format!("argument {}", no).as_str())?);
        it.eat_if(Token::Comma);
        Ok(())
    })?;

    Ok(arguments)
}

fn parse_postfix_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let name_or_arg = it.parse(&parse_expression, "method name or function argument")?;
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);

    it.peek(
        &|it, token_pos| {
            let pre = pre.clone();
            let name_or_arg = name_or_arg.clone();
            match token_pos {
                _ if is_start_expression_exclude_unary(token_pos) =>
                    match parse_postfix_call(*name_or_arg.clone(), it) {
                        Ok(right) => {
                            let (en_line, en_pos) = (right.en_line, right.en_pos);
                            let node = ASTNode::Call { left: Box::from(pre), right };
                            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
                        }
                        err => err
                    },
                _ => {
                    let (en_line, en_pos) = (name_or_arg.en_line, name_or_arg.en_pos);
                    let node = ASTNode::Call { left: Box::from(pre), right: name_or_arg };
                    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
                }
            }
        },
        {
            let (en_line, en_pos) = (name_or_arg.en_line, name_or_arg.en_pos);
            let node = ASTNode::Call { left: Box::from(pre.clone()), right: name_or_arg.clone() };
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        },
        "postfix call"
    )
}
