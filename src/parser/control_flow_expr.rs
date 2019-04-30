use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::collection::parse_one_or_more_expr;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::common::start_pos;
use crate::parser::TPIterator;

macro_rules! get_one_or_more {
    ($it:expr, $msg:expr) => {{
        match parse_one_or_more_expr($it, $msg) {
            Ok(node) => node,
            Err(err) => return Err(err)
        }
    }};
}

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
    let cond: Vec<ASTNodePos> = get_one_or_more!(it, "if condition");
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

    let cond: Vec<ASTNodePos> = get_one_or_more!(it, "match expression");
    check_next_is!(it, Token::With);
    check_next_is!(it, Token::NL);
    let cases: Vec<ASTNodePos> = get_or_err_direct!(it, parse_match_cases, "match cases");

    let (en_line, en_pos) = match (cond.last(), cases.last()) {
        (_, Some(ast_node_pos)) => (ast_node_pos.en_line, ast_node_pos.en_pos),
        (Some(cond), _) => (cond.en_line, cond.en_pos),
        _ => (st_line, st_pos)
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
            _ => cases.push(get_or_err_direct!(it, parse_match_case, "when case"))
        }
    }

    Ok(cases)
}

fn parse_match_case(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "match case");
    check_next_is!(it, Token::BTo);
    let expr_or_stmt: Box<ASTNodePos> = get_or_err!(it, parse_expr_or_stmt, "then");

    Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: expr_or_stmt.en_line,
        en_pos: expr_or_stmt.en_pos,
        node: ASTNode::Case { cond, body: expr_or_stmt }
    })
}
