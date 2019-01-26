use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::control_flow_expr::parse_when_cases;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::statement::parse_statement;
use crate::parser::TPIterator;

pub fn parse_expr_or_stmt(it: &mut TPIterator) -> ParseResult {
    if let Some(TokenPos { token: Token::NL, .. }) = it.peek() {
        it.next();
        return Ok(get_or_err_direct!(it, parse_block, "expression or statement"));
    }

    let (st_line, st_pos) = start_pos(it);
    let expr_or_stmt: Box<ASTNodePos> = match it.peek() {
        Some(TokenPos { token: Token::Def, .. }) |
        Some(TokenPos { token: Token::Mut, .. }) |
        Some(TokenPos { token: Token::Print, .. }) |
        Some(TokenPos { token: Token::PrintLn, .. }) |
        Some(TokenPos { token: Token::For, .. }) |
        Some(TokenPos { token: Token::While, .. }) |
        Some(TokenPos { token: Token::Retry, .. }) |
        Some(TokenPos { token: Token::Type, .. }) =>
            get_or_err!(it, parse_statement, "expression or statement"),
        _ => get_or_err!(it, parse_expression, "expression or statement")
    };

    return match it.peek() {
        Some(TokenPos { token: Token::Raises, .. }) => {
            it.next();
            check_next_is!(it, Token::LSBrack);

            let mut errors: Vec<ASTNodePos> = Vec::new();
            loop {
                match it.next() {
                    Some(TokenPos { token: Token::Comma, .. }) => {
                        it.next();
                        errors.push(get_or_err_direct!(it, parse_id, "raises"))
                    }
                    Some(TokenPos { token: Token::RSBrack, .. }) => break,
                    Some(tp) =>
                        return Err(TokenErr { expected: Token::RSBrack, actual: tp.clone() }),
                    None => return Err(EOFErr { expected: Token::RSBrack })
                }
            }

            return Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: 0,
                en_pos: 0,
                node: ASTNode::Raises { expr_or_stmt, errors },
            });
        }
        Some(TokenPos { token: Token::Handle, .. }) => {
            it.next();
            check_next_is!(it, Token::When);

            let cases = get_or_err!(it, parse_when_cases, "handle cases");
            return Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: 0,
                en_pos: 0,
                node: ASTNode::Handle { expr_or_stmt, cases },
            });
        }
        Some(TokenPos { token: Token::If, .. }) => {
            it.next();
            let cond: Box<ASTNodePos> = get_or_err!(it, parse_expression, "postfix if");
            while let Some(TokenPos { token: Token::NL, .. }) = it.peek() { it.next(); }

            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: cond.en_line,
                en_pos: cond.en_pos,
                node: ASTNode::If { cond, then: expr_or_stmt },
            })
        }
        _ => Ok(*expr_or_stmt)
    };
}
