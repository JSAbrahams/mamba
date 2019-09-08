use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected;
use crate::parser::parse_result::ParseResult;

pub fn parse_statements(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let start = it.start_pos("block")?;
    let mut statements: Vec<ASTNodePos> = Vec::new();

    it.peek_while_not_token(&Token::Dedent, &mut |it, token_pos| match &token_pos.token {
        Token::NL => {
            it.eat(&Token::NL, "block")?;
            Ok(())
        }
        Token::Comment(comment) => {
            let end = it.eat(&Token::Comment(comment.clone()), "block")?;
            let node = ASTNode::Comment { comment: comment.clone() };
            statements.push(ASTNodePos::new(&token_pos.start, &end, node));
            Ok(())
        }
        _ => {
            statements.push(*it.parse(&parse_expr_or_stmt, "block", &start)?);
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
    let start = it.start_pos("block")?;
    it.eat(&Token::Indent, "block")?;
    let statements = it.parse_vec(&parse_statements, "block", &start)?;
    let end = statements.last().cloned().map_or(start.clone(), |stmt| stmt.position.end);

    it.eat(&Token::Dedent, "block")?;
    Ok(Box::from(ASTNodePos::new(&start, &end, ASTNode::Block { statements })))
}
