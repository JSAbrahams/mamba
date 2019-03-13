use crate::lexer::token::Token;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_statements(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut stmts: Vec<ASTNodePos> = Vec::new();
    while let Some(&t) = it.peek() {
        match t.token {
            Token::Dedent | Token::Class | Token::Type => break,
            Token::NL => { it.next(); }
            _ => stmts.push(get_or_err_direct!(it, parse_expr_or_stmt, "block"))
        }
    }

    return Ok(stmts);
}

pub fn parse_block(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Indent);

    let statements: Vec<ASTNodePos> = get_or_err_direct!(it, parse_statements, "block");
    let (en_line, en_pos) = match statements.last() {
        Some(stmt) => (stmt.en_line, stmt.en_pos),
        None => (st_line, st_pos)
    };

    if it.peek().is_some() { check_next_is!(it, Token::Dedent); }
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Block { statements } })
}
