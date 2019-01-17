use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::declaration::parse_declaration;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;
use std::env;

pub fn parse_statement(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    return match it.peek() {
        Some(TokenPos { token: Token::Print, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "statement");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: expr.en_line,
                en_pos: expr.en_pos,
                node: ASTNode::Print { expr },
            })
        }

        Some(TokenPos { token: Token::Let, .. }) | Some(TokenPos { token: Token::Mut, .. }) =>
            parse_declaration(it),

        Some(TokenPos { token: Token::For, .. }) | Some(TokenPos { token: Token::While, .. }) =>
            parse_cntrl_flow_stmt(it),

        Some(&next) => Err(CustomErr { expected: "statement".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "statement".to_string() })
    };
}
