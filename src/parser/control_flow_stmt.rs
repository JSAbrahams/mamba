use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_cntrl_flow_stmt(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    match it.peek() {
        Some(TokenPos { token: Token::While, .. }) => parse_while(it),
        Some(TokenPos { token: Token::For, .. }) => parse_for(it),
        Some(TokenPos { token: Token::Break, .. }) => {
            let (en_line, en_pos) = it.end_pos()?;
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Break })
        }
        Some(TokenPos { token: Token::Continue, .. }) => {
            let (en_line, en_pos) = it.end_pos()?;
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Continue })
        }

        Some(&next) => Err(CustomErr {
            expected: "control flow statement".to_string(),
            actual:   next.clone()
        }),
        None => Err(CustomEOFErr { expected: "control flow statement".to_string() })
    }
}

fn parse_while(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    it.eat(Token::While);
    let cond: Box<ASTNodePos> = it.parse(parse_expression, "while condition");
    it.eat(Token::Do);
    let body: Box<ASTNodePos> = it.parse(parse_expr_or_stmt, "while body");

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::While { cond, body };

    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

fn parse_for(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    it.eat(Token::For);
    let expr: Box<ASTNodePos> = it.parse(parse_expression, "for expression");
    it.eat(Token::Do);
    let body: Box<ASTNodePos> = it.parse(parse_expr_or_stmt, "for body");

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::For { expr, body };

    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}
