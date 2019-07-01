use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected;
use crate::parser::parse_result::ParseResult;

pub fn parse_statements(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut statements: Vec<ASTNodePos> = Vec::new();
    it.peek_while_not_token(&Token::Dedent, &mut |it, token_pos| match &token_pos.token {
        Token::NL => {
            it.eat(&Token::NL, "block")?;
            Ok(())
        }
        Token::Comment(comment) => {
            let (st_line, st_pos) = (token_pos.st_line, token_pos.st_pos);
            it.eat(&Token::Comment(comment.clone()), "block")?;
            let (en_line, en_pos) = (st_line, Token::Comment(comment.clone()).width());
            let node = ASTNode::Comment { comment: comment.clone() };
            statements.push(ASTNodePos { st_line, st_pos, en_line, en_pos, node });
            Ok(())
        }
        _ => {
            statements.push(*it.parse(&parse_expr_or_stmt, "block")?);
            let invalid = |token_pos: &TokenPos| {
                token_pos.token != Token::NL && token_pos.token != Token::Dedent
            };
            if it.peak_if_fn(&invalid) {
                return Err(expected(&Token::NL, &token_pos.clone(), "block"));
            }
            Ok(())
        }
    })?;

    Ok(statements)
}

pub fn parse_block(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("block")?;
    it.eat(&Token::Indent, "block")?;

    let statements = it.parse_vec(&parse_statements, "block")?;
    let (en_line, en_pos) = match statements.last() {
        Some(stmt) => (stmt.en_line, stmt.en_pos),
        None => (st_line, st_pos)
    };

    it.eat(&Token::Dedent, "block")?;
    let node = ASTNode::Block { statements };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}
