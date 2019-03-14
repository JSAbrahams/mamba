use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::end_pos;
use crate::parser::expression::is_start_expression;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::ast_node::ASTNode;
use crate::parser::ast_node::ASTNodePos;
use crate::parser::TPIterator;

pub fn parse_reassignment(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::Assign);
    let right: Box<ASTNodePos> = get_or_err!(it, parse_expression, "reassignment");

    let (en_line, en_pos) = (right.en_line, right.en_pos);
    let node = ASTNode::ReAssign { left: Box::new(pre), right };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_anon_fun(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::To);
    let body: Box<ASTNodePos> = get_or_err!(it, parse_expression, "anonymous function");

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::AnonFun { args: Box::new(pre), body };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::Point, .. }) => parse_regular_call(false, pre, it),
        Some(TokenPos { token: Token::LRBrack, .. }) => parse_direct_call(pre, it),
        Some(&tp) if is_start_expression(tp.clone()) => parse_postfix_call(pre, it),
        Some(&tp) => {
            Err(CustomErr { expected: String::from("function call"), actual: tp.clone() })
        }
        None => Err(CustomEOFErr { expected: String::from("function call") })
    }
}

fn parse_direct_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);
    let args = match parse_arguments(it, "arguments") {
        Ok(args) => args,
        Err(err) => return Err(err)
    };

    let (en_line, en_pos) = end_pos(it);
    check_next_is!(it, Token::RRBrack);

    let node = ASTNode::FunctionCallDirect { name: Box::from(pre), args };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

fn parse_regular_call(fun: bool, pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);
    it.next();
    let name: Box<ASTNodePos> = get_or_err!(it, parse_id, "call name");

    let args: Vec<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::LRBrack, .. }) => match parse_arguments(it, "arguments") {
            Ok(args) => {
                check_next_is!(it, Token::RRBrack);
                args
            }
            Err(err) => return Err(err)
        },
        Some(&tp) if is_start_expression(tp.clone()) => match parse_expressions(it, "arguments") {
            Ok(args) => {
                if args.len() > 1 {
                    return Err(InternalErr { message: format!("Postfix notation only possible \
                                                               with 1 argument,but {} were given.",
                                                              args.len()) });
                }
                args
            }
            Err(err) => return Err(err)
        },
        _ => vec![]
    };

    let node = if fun {
        ASTNode::FunctionCall { namespace: Box::from(pre), name, args }
    } else {
        ASTNode::MethodCall { instance: Box::from(pre), name, args }
    };

    Ok(ASTNodePos { st_line, st_pos, en_line: 0, en_pos: 0, node })
}

fn parse_arguments(it: &mut TPIterator, msg: &str) -> ParseResult<Vec<ASTNodePos>> {
    check_next_is!(it, Token::LRBrack);
    let mut arguments = Vec::new();
    let mut pos = 0;

    while let Some(&t) = it.peek() {
        match t.token {
            Token::RRBrack => break,
            _ => {
                arguments.push(get_or_err_direct!(it,
                                                  parse_expression,
                                                  String::from(msg)
                                                  + " (pos "
                                                  + &pos.to_string()
                                                  + ")"));
                if let Some(&t) = it.peek() {
                    if t.token != Token::RRBrack {
                        check_next_is!(it, Token::Comma);
                    }
                }
            }
        }
        pos += 1;
    }

    Ok(arguments)
}

fn parse_postfix_call(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let name_or_arg = get_or_err!(it, parse_expression, "method name or function argument");
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);

    let (en_line, en_pos, node) = match it.peek() {
        Some(&tp) if is_start_expression(tp.clone()) => {
            match parse_postfix_call(*name_or_arg, it) {
                Ok(post) => (post.en_line,
                             post.en_pos,
                             ASTNode::Call { instance_or_met: Box::from(pre),
                                             met_or_arg:      Box::from(post) }),
                err => return err
            }
        }
        _ => (name_or_arg.en_line,
              name_or_arg.en_pos,
              ASTNode::Call { instance_or_met: Box::from(pre),
                              met_or_arg:      name_or_arg })
    };

    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

fn parse_expressions(it: &mut TPIterator, msg: &str) -> ParseResult<Vec<ASTNodePos>> {
    let mut expressions = Vec::new();
    let mut pos = 0;

    while let Some(&t) = it.peek() {
        if is_start_expression(t.clone()) {
            expressions.push(get_or_err_direct!(it,
                                                parse_expression,
                                                String::from(msg)
                                                + " (pos "
                                                + &pos.to_string()
                                                + ")"));
        } else {
            break;
        }
        pos += 1;
    }

    Ok(expressions)
}
