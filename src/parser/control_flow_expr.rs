use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_cntrl_flow_expr(it: &mut TPIterator) -> ParseResult {
    return match it.peek() {
        Some(TokenPos { token: Token::If, .. }) => parse_if(it),
        Some(TokenPos { token: Token::When, .. }) => parse_when(it),

        Some(&next) => Err(CustomErr {
            expected: "control flow expression".to_string(),
            actual: next.clone(),
        }),
        None => Err(CustomEOFErr { expected: "control flow expression".to_string() })
    };
}

fn parse_if(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::If);
    let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "if condition");
    check_next_is!(it, Token::Then);
    let then: Box<ASTNodePos> = get_or_err!(it, parse_expr_or_stmt, "if then branch");

    if let Some(&&TokenPos { token: Token::Else, .. }) = it.peek() {
        it.next();
        let _else = get_or_err!(it, parse_expr_or_stmt, "if else branch");
        Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line: _else.en_line,
            en_pos: _else.en_pos,
            node: ASTNode::IfElse { cond, then, _else },
        })
    } else {
        Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line: then.en_line,
            en_pos: then.en_pos,
            node: ASTNode::If { cond, then },
        })
    }
}

fn parse_when(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::When);

    let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "when expression");
    check_next_is!(it, Token::NL);

    panic!("not implemented")
}

fn parse_when_case(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "when case");
    check_next_is!(it, Token::Then);
    let then: Box<ASTNodePos> = get_or_err!(it, parse_expr_or_stmt, "then");

    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: then.en_line,
        en_pos: then.en_pos,
        node: ASTNode::If { cond, then },
    });
}
