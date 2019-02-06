use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::collection::parse_zero_or_more_expr;
use crate::parser::end_pos;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_reassignment(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Assign);
    let right: Box<ASTNodePos> = get_or_err!(it, parse_expression, "reassignment");

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: right.en_line,
        en_pos: right.en_pos,
        node: ASTNode::ReAssign { left: Box::new(pre), right },
    });
}

pub fn parse_anon_fun(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::To);
    let body: Box<ASTNodePos> = get_or_err!(it, parse_expression, "anonymous function");

    Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: body.en_line,
        en_pos: body.en_pos,
        node: ASTNode::AnonFun { args: Box::new(pre), body },
    })
}

pub fn parse_function_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    unimplemented!("function call needs to be rewritten.");

    let (st_line, st_pos) = start_pos(it);
    let next_name;
    if let Some(TokenPos { token: Token::Point, .. }) = it.peek() {
        it.next();
        next_name = true;
    } else {
        if let Some(TokenPos { token: Token::DDoublePoint, .. }) = it.peek() {
            it.next();
            next_name = true;
        }
    }

    let name: Option<Box<ASTNodePos>>;
    if next_name {
        name = Some(get_or_err_direct!(it, parse_id, "function or method name"))
    } else { name = None }

    let brackets;
    if let Some(TokenPos { token: Token::LRBrack, .. }) = it.peek() {
        it.next();
        brackets = true;
    } else { brackets = false; }

    let args: Vec<ASTNodePos>;
    if brackets {
        args = match parse_zero_or_more_expr(it, "function call") {
            Ok(a) => a,
            Err(err) => return Err(err)
        };
    } else {
        args = match parse_expression(it) {
            Ok(a) => vec![a],
            Err(err) => return Err(err)
        };
    }

    let (en_line, en_pos) = if brackets {
        let pos = end_pos(it);
        check_next_is!(it, Token::RRBrack);
        pos
    } else {
        match args.last() {
            Some(arg) => (arg.en_line, arg.en_pos),
            None => (pre.en_line, pre.en_pos)
        }
    };

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line,
        en_pos,
        node: ASTNode::FunCall {
            instance_or_namespace: Box::from(pre),
            fun: Box::from(pre),
            args,
        },
    });
}
