use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_generics;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::block::parse_block;
use crate::parser::control_flow_expr::parse_match_cases;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseResult;
use crate::parser::statement::parse_statement;

pub fn parse_expr_or_stmt(it: &mut TPIterator) -> ParseResult {
    if let Some(TokenPos { token: Token::NL, .. }) = it.peek() {
        it.next();
        return Ok(it.parse(parse_block, "expression or statement"));
    }

    let result = match it.peek() {
        Some(TokenPos { token: Token::Def, .. })
        | Some(TokenPos { token: Token::Mut, .. })
        | Some(TokenPos { token: Token::Print, .. })
        | Some(TokenPos { token: Token::For, .. })
        | Some(TokenPos { token: Token::While, .. })
        | Some(TokenPos { token: Token::Retry, .. })
        | Some(TokenPos { token: Token::Pass, .. })
        | Some(TokenPos { token: Token::Raise, .. })
        | Some(TokenPos { token: Token::With, .. }) => parse_statement(it),
        _ => parse_expression(it)
    };

    match (result, it.peek()) {
        (Ok(pre), Some(TokenPos { token: Token::Raises, .. })) => parse_raise(pre, it),
        (Ok(pre), Some(TokenPos { token: Token::Handle, .. })) => parse_handle(pre, it),
        (result, _) => result
    }
}

pub fn parse_raise(expr_or_stmt: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Raises);

    let errors: Vec<ASTNodePos> = it.parse(parse_generics, "raises");

    let node = ASTNode::Raises { expr_or_stmt: Box::from(expr_or_stmt), errors };
    Ok(ASTNodePos { st_line, st_pos, en_line: 0, en_pos: 0, node })
}

pub fn parse_handle(expr_or_stmt: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Handle);

    it.eat(Token::NL);
    let cases = it.parse(parse_match_cases, "handle cases");

    let node = ASTNode::Handle { expr_or_stmt: Box::from(expr_or_stmt), cases };
    Ok(ASTNodePos { st_line, st_pos, en_line: 0, en_pos: 0, node })
}
