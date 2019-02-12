use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::expression::is_expression;
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

pub fn parse_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::Point, .. }) => parse_regular_call(false, pre, it),
        Some(TokenPos { token: Token::DDoublePoint, .. }) => parse_regular_call(true, pre, it),
        Some(TokenPos { token: Token::LRBrack, .. }) => parse_direct_call(pre, it),
        Some(&tp) if is_expression(tp.clone()) => parse_postfix_call(pre, it),
        Some(&tp) => return Err(CustomErr {
            expected: String::from("function call"),
            actual: tp.clone(),
        }),
        None => return Err(CustomEOFErr { expected: String::from("function call") })
    }
}

fn parse_direct_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let namespace = Box::from(ASTNodePos {
        st_line: pre.st_line,
        st_pos: pre.st_pos,
        en_line: pre.en_line,
        en_pos: pre.en_pos,
        node: ASTNode::_Self,
    });

    let args = match parse_arguments(it, "arguments") {
        Ok(args) => args,
        Err(err) => return Err(err)
    };

    let (en_line, en_pos) = end_pos(it);
    check_next_is!(it, Token::RRBrack);

    return Ok(ASTNodePos {
        st_line: pre.st_line,
        st_pos: pre.st_pos,
        en_line,
        en_pos,
        node: ASTNode::FunCall {
            namespace,
            name: Box::from(pre),
            args,
        },
    });
}

fn parse_regular_call(fun: bool, pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    it.next();
    let name: Box<ASTNodePos> = get_or_err!(it, parse_id, "call name");

    let args: Vec<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::LRBrack, .. }) =>
            match parse_arguments(it, "arguments") {
                Ok(args) => {
                    check_next_is!(it, Token::RRBrack);
                    args
                }
                Err(err) => return Err(err)
            }
        Some(&tp) if is_expression(tp.clone()) =>
            match parse_expressions(it, "arguments") {
                Ok(args) => {
                    if args.len() > 1 {
                        return Err(InternalErr {
                            message: format!("Postfix notation only possible with 1 argument,\
                    but {} were given.", args.len())
                        });
                    }
                    args
                }
                Err(err) => return Err(err)
            }
        _ => vec![]
    };

    return Ok(ASTNodePos {
        st_line: pre.st_line,
        st_pos: pre.st_pos,
        en_line: 0,
        en_pos: 0,
        node: if fun {
            ASTNode::FunCall { namespace: Box::from(pre), name, args }
        } else {
            ASTNode::MetCall { instance: Box::from(pre), name, args }
        },
    });
}

fn parse_arguments(it: &mut TPIterator, msg: &str) -> ParseResult<Vec<ASTNodePos>> {
    check_next_is!(it, Token::LRBrack);
    let mut arguments = Vec::new();
    let mut pos = 0;

    while let Some(&t) = it.peek() {
        match t.token {
            Token::RRBrack => break,
            _ => {
                arguments.push(get_or_err_direct!(it, parse_expression,
                                  String::from(msg) + " (pos "+ &pos.to_string() + ")"));
                if let Some(&t) = it.peek() {
                    if t.token != Token::RRBrack { check_next_is!(it, Token::Comma); }
                }
            }
        }
        pos += 1;
    }

    return Ok(arguments);
}

fn parse_postfix_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let name_or_arg = get_or_err!(it, parse_expression, "method name or function argument");

    return Ok(ASTNodePos {
        st_line: pre.st_line,
        st_pos: pre.st_pos,
        en_line: name_or_arg.en_line,
        en_pos: name_or_arg.en_pos,
        node: ASTNode::Call {
            instance_or_met: Box::from(pre),
            met_or_arg: Box::from(name_or_arg),
        },
    });
}

fn parse_expressions(it: &mut TPIterator, msg: &str) -> ParseResult<Vec<ASTNodePos>> {
    let mut expressions = Vec::new();
    let mut pos = 0;

    while let Some(&t) = it.peek() {
        if is_expression(t.clone()) {
            expressions.push(get_or_err_direct!(it, parse_expression,
                                  String::from(msg) + " (pos "+ &pos.to_string() + ")"));
        } else { break; }
        pos += 1;
    }

    return Ok(expressions);
}
