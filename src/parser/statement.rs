use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::definition::parse_definition;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_statement(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    match it.peek() {
        Some(TokenPos { token: Token::Print, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = it.parse(parse_expression, "print");

            let (en_line, en_pos) = (expr.en_line, expr.en_pos);
            let node = ASTNode::Print { expr };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        Some(TokenPos { token: Token::Pass, .. }) => {
            let (en_line, en_pos) = it.end_pos()?;
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Pass })
        }
        Some(TokenPos { token: Token::Retry, .. }) => {
            let (en_line, en_pos) = it.end_pos()?;
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Retry })
        }
        Some(TokenPos { token: Token::Raise, .. }) => {
            let (en_line, en_pos) = it.end_pos()?;
            it.next();
            let error = it.parse(parse_expression, "raise");
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Raise { error } })
        }

        Some(TokenPos { token: Token::Def, .. }) => parse_definition(it),
        Some(TokenPos { token: Token::With, .. }) => parse_with(it),

        Some(TokenPos { token: Token::For, .. }) | Some(TokenPos { token: Token::While, .. }) =>
            parse_cntrl_flow_stmt(it),

        Some(&next) => Err(CustomErr { expected: "statement".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "statement".to_string() })
    }
}

pub fn parse_with(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::With);
    let resource = it.parse(parse_expression, "with resource");
    let _as = if let Some(TokenPos { token: Token::As, .. }) = it.peek() {
        it.next();
        Some(it.parse(parse_id_maybe_type, "with id"))
    } else {
        None
    };
    let expr: Box<ASTNodePos> = it.parse(parse_expr_or_stmt, "with body");

    let (en_line, en_pos) = (expr.en_line, expr.en_pos);
    let node = ASTNode::With { resource, _as, expr };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}
