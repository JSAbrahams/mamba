use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_type_def;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::definition::parse_definition;
use crate::parser::end_pos;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_statement(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    return match it.peek() {
        Some(TokenPos { token: Token::Print, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "print");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: expr.en_line,
                en_pos: expr.en_pos,
                node: ASTNode::Print { expr },
            })
        }
        Some(TokenPos { token: Token::PrintLn, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "print line");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: expr.en_line,
                en_pos: expr.en_pos,
                node: ASTNode::PrintLn { expr },
            })
        }
        Some(TokenPos { token: Token::Ensure, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "ensure");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: expr.en_line,
                en_pos: expr.en_pos,
                node: ASTNode::Ensure { expr },
            })
        }
        Some(TokenPos { token: Token::Require, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "require");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: expr.en_line,
                en_pos: expr.en_pos,
                node: ASTNode::Require { expr },
            })
        }
        Some(TokenPos { token: Token::Retry, .. }) => {
            let (en_line, en_pos) = end_pos(it);
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Retry })
        }

        Some(TokenPos { token: Token::Def, .. }) => parse_definition(it),
        Some(TokenPos { token: Token::For, .. }) | Some(TokenPos { token: Token::While, .. }) =>
            parse_cntrl_flow_stmt(it),

        Some(TokenPos { token: Token::Type, .. }) => parse_type_def(it),

        Some(&next) => Err(CustomErr { expected: "statement".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "statement".to_string() })
    };
}
