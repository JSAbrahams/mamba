use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::end_pos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::TPIterator;

pub fn parse_cntrl_flow_stmt(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    return match it.peek() {
        Some(TokenPos { token: Token::While, .. }) => parse_while(it),
        Some(TokenPos { token: Token::For, .. }) => parse_for(it),
        Some(TokenPos { token: Token::Break, .. }) => {
            let (en_line, en_pos) = end_pos(it);
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Break })
        }
        Some(TokenPos { token: Token::Continue, .. }) => {
            let (en_line, en_pos) = end_pos(it);
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Continue })
        }

        Some(&next) => {
            return Err(CustomErr { expected: "control flow statement".to_string(),
                                   actual:   next.clone() })
        }
        None => return Err(CustomEOFErr { expected: "control flow statement".to_string() })
    };
}

fn parse_while(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::While);
    let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "while condition");
    check_next_is!(it, Token::Do);
    let body: Box<ASTNodePos> = get_or_err!(it, parse_expr_or_stmt, "while body");

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::While { cond, body };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

fn parse_for(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    check_next_is!(it, Token::For);
    let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "for expression");
    check_next_is!(it, Token::In);
    let collection: Box<ASTNodePos> = get_or_err!(it, parse_expression, "for collection");
    check_next_is!(it, Token::Do);
    let body: Box<ASTNodePos> = get_or_err!(it, parse_expr_or_stmt, "for body");

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::For { expr, collection, body };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}
