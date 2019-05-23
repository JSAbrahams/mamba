use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::common::start_pos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;
use crate::parser::_type::parse_type;

pub fn parse_cntrl_flow_expr(it: &mut TPIterator) -> ParseResult {
    match it.peek() {
        Some(TokenPos { token: Token::If, .. }) => parse_if(it),
        Some(TokenPos { token: Token::Match, .. }) => parse_match(it),

        Some(&next) => Err(CustomErr {
            expected: "control flow expression".to_string(),
            actual:   next.clone()
        }),
        None => Err(CustomEOFErr { expected: "control flow expression".to_string() })
    }
}

fn parse_if(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::If);
    let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "if condition");
    check_next_is!(it, Token::Then);
    let then: Box<ASTNodePos> = get_or_err!(it, parse_expr_or_stmt, "if then branch");

    let _else = if let Some(&&TokenPos { token: Token::Else, .. }) = it.peek() {
        it.next();
        Some(get_or_err!(it, parse_expr_or_stmt, "if else branch"))
    } else {
        None
    };

    let (en_line, en_pos) = (then.en_line, then.en_pos);
    let node = ASTNode::IfElse { cond, then, _else };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

fn parse_match(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Match);

    let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "match expression");
    check_next_is!(it, Token::NL);
    let cases: Vec<ASTNodePos> = get_or_err_direct!(it, parse_match_cases, "match cases");

    let (en_line, en_pos) = match (&cond, cases.last()) {
        (_, Some(ast_node_pos)) => (ast_node_pos.en_line, ast_node_pos.en_pos),
        (cond, _) => (cond.en_line, cond.en_pos)
    };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Match { cond, cases } })
}

pub fn parse_match_cases(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    check_next_is!(it, Token::Indent);

    let mut cases = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::NL => {
                it.next();
            }
            Token::Dedent => {
                it.next();
                break;
            }
            _ => cases.push(get_or_err_direct!(it, parse_match_case, "match case"))
        }
    }

    Ok(cases)
}

fn parse_match_case(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression_maybe_type, "match case");
    check_next_is!(it, Token::BTo);
    let body: Box<ASTNodePos> = get_or_err!(it, parse_expr_or_stmt, "then");

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::Case { cond, body };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_expression_maybe_type(it: &mut TPIterator) -> ParseResult {
    let mutable;
    if it.peek().is_some() && it.peek().unwrap().token == Token::Mut {
        mutable = true;
        it.next();
    } else {
        mutable = false;
    }

    let id: Box<ASTNodePos> = get_or_err!(it, parse_expression, "id maybe type");
    let (en_line, en_pos, _type) = match it.peek() {
        Some(TokenPos { token: Token::DoublePoint, .. }) => {
            it.next();
            let _type: Box<ASTNodePos> = get_or_err!(it, parse_type, "id type");
            (_type.en_line, _type.en_pos, Some(_type))
        }
        _ => (id.en_line, id.en_pos, None)
    };

    let (st_line, st_pos) = (id.st_line, id.st_pos);
    let node = ASTNode::IdType { id, mutable, _type };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}
