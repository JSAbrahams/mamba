use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast_node::ASTNode;
use crate::parser::ast_node::ASTNodePos;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::definition::parse_definition;
use crate::parser::end_pos;
use crate::parser::expr_or_stmt::parse_handle;
use crate::parser::expr_or_stmt::parse_raise;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_statement(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    let result = match it.peek() {
        Some(TokenPos { token: Token::Print, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "print");

            let (en_line, en_pos) = (expr.en_line, expr.en_pos);
            let node = ASTNode::Print { expr };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        Some(TokenPos { token: Token::Retry, .. }) => {
            let (en_line, en_pos) = end_pos(it);
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Retry })
        }

        Some(TokenPos { token: Token::Def, .. }) => parse_definition(it),
        Some(TokenPos { token: Token::For, .. }) | Some(TokenPos { token: Token::While, .. }) =>
            parse_cntrl_flow_stmt(it),

        Some(&next) => Err(CustomErr { expected: "statement".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "statement".to_string() })
    };

    match (result, it.peek()) {
        (Ok(pre), Some(TokenPos { token: Token::Raises, .. })) => parse_raise(pre, it),
        (Ok(pre), Some(TokenPos { token: Token::Handle, .. })) => parse_handle(pre, it),
        (result, _) => result
    }
}
