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

pub fn parse_reassignment(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Assign);

    let right = it.parse(&parse_expression, "reassignment")?;

    let (en_line, en_pos) = (right.en_line, right.en_pos);
    let node = ASTNode::Reassign { left: Box::new(pre), right };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_anon_fun(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::BSlash);

    let mut args: Vec<ASTNodePos> = vec![];
    let mut pos = 1;
    it.while_some_and_not(Token::BTo, &|_| {
        args.push(*it.parse(
            &parse_id_maybe_type,
            format!("anonymous function arg (pos {})", pos).as_str()
        )?);

        it.eat_if(Token::Comma);
        pos += 1;
        Ok(())
    });

    it.eat(Token::BTo);

    let body = it.parse(&parse_expression, "anonymous function body")?;
    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::AnonFun { args, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    it.peek(
        &|token_pos| match token_pos {
            TokenPos { token: Token::Point, .. } => parse_regular_call(pre, it),
            TokenPos { token: Token::LRBrack, .. } => parse_direct_call(pre, it),
            tp if is_start_expression_exclude_unary(tp) => parse_postfix_call(pre, it),
            tp => Err(CustomErr { expected: String::from("function call"), actual: tp.clone() })
        },
        CustomEOFErr { expected: String::from("function call") }
    )
}

fn parse_direct_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);
    let args = it.parse_vec(&parse_arguments, "arguments")?;
    let (en_line, en_pos) = it.end_pos()?;
    it.eat(Token::RRBrack);

    let node = ASTNode::DirectCall { name: Box::from(pre), args };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_regular_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);
    it.eat(Token::Point);
    let name = it.parse(&parse_id, "call name")?;
    let mut method = true;

    let args = it.peek_vec_or(
        &|token_pos| match token_pos {
            TokenPos { token: Token::LRBrack, .. } => it.parse_vec(&parse_arguments, "arguments"),
            token_pos if is_start_expression_exclude_unary(token_pos) =>
                Ok(vec![*it.parse(&parse_expression, "postfix arg")?]),
            _ => {
                method = false;
                Ok(vec![])
            }
        },
        Ok(vec![])
    )?;

    let node = if method {
        ASTNode::MethodCall { instance: Box::from(pre), name, args }
    } else {
        ASTNode::Call { left: Box::from(pre), right: name }
    };

    let (en_line, en_pos) = if let Some(ast_node) = args.last() {
        (ast_node.en_line, ast_node.en_pos)
    } else {
        (name.en_line, name.en_pos)
    };

    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_arguments(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(Token::LRBrack);

    let mut arguments = Vec::new();
    it.while_some_and_not(Token::RRBrack, &|_| {
        arguments.push(*it.parse(&parse_expression, "argument")?);
        it.eat_if(Token::Comma);
        Ok(())
    });

    it.eat(Token::RRBrack);
    Ok(arguments)
}

fn parse_postfix_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let name_or_arg = it.parse(&parse_expression, "method name or function argument")?;
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);

    it.peek_or(
        &|token_pos| match token_pos {
            tp if is_start_expression_exclude_unary(tp) =>
                match parse_postfix_call(*name_or_arg, it) {
                    Ok(post) => {
                        let node = ASTNode::Call { left: Box::from(pre), right: Box::from(post) };
                        let (en_line, en_pos) = it.end_pos()?;
                        Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
                    }
                    err => return err
                },
            _ => {
                let node = ASTNode::Call { left: Box::from(pre), right: name_or_arg };
                let (en_line, en_pos) = it.end_pos()?;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
        },
        {
            let node = ASTNode::Call { left: Box::from(pre), right: name_or_arg };
            let (en_line, en_pos) = (name_or_arg.en_line, name_or_arg.en_pos);
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        }
    )
}
