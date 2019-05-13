use crate::lexer::token::Token;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::common::start_pos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;

pub fn parse_statements(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut statements: Vec<ASTNodePos> = Vec::new();
    while let Some(&t) = it.peek() {
        match &t.token {
            Token::Dedent => {
                it.next();
                break;
            },
            Token::NL => {
                it.next();
            }
            Token::Comment(comment) => {
                let node = ASTNode::Comment { comment: comment.clone() };
                let node_pos = ASTNodePos {
                    st_line: t.line,
                    st_pos: t.pos,
                    en_line: t.line,
                    en_pos: t.pos + comment.len() as i32,
                    node
                };
                statements.push(node_pos);
                it.next();
            }
            _ => {
                statements.push(get_or_err_direct!(it, parse_expr_or_stmt, "block"));
                if let Some(&t) = it.peek() {
                    if t.token != Token::NL && t.token != Token::Dedent {
                        return Err(TokenErr { expected: Token::NL, actual: t.clone() });
                    }
                }
            }
        }
    }

    Ok(statements)
}

pub fn parse_block(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Indent);

    let statements: Vec<ASTNodePos> = get_or_err_direct!(it, parse_statements, "block");

    let (en_line, en_pos) = match statements.last() {
        Some(stmt) => (stmt.en_line, stmt.en_pos),
        None => (st_line, st_pos)
    };

    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Block { statements } })
}
