use crate::lexer::token::Token;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_cntrl_flow_expr(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::If => parse_if(it),
            Token::Match => parse_match(it),
            _ => Err(expected_one_of(
                &[Token::If, Token::Match],
                token_pos,
                "control flow expression"
            ))
        },
        &[Token::If, Token::Match],
        "control flow expression"
    )
}

fn parse_if(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("if expression")?;
    it.eat(&Token::If, "if expressions")?;
    let cond = it.parse(&parse_expression, "if expression", start)?;
    it.eat(&Token::Then, "if expression")?;
    let then = it.parse(&parse_expr_or_stmt, "if expression", start)?;
    let _else = it.parse_if(&Token::Else, &parse_expr_or_stmt, "if else branch", start)?;

    let node = ASTNode::IfElse { cond, then, _else };
    Ok(Box::from(ASTNodePos::new(start, then.position.end, node)))
}

fn parse_match(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("match")?;
    it.eat(&Token::Match, "match")?;
    let cond = it.parse(&parse_expression, "match", start)?;
    it.eat(&Token::NL, "match")?;
    let cases = it.parse_vec(&parse_match_cases, "match", start)?;
    let end = match (&cond, cases.last()) {
        (_, Some(last_case)) => last_case.position.end,
        (cond, _) => cond.position.end
    };

    let node = ASTNode::Match { cond, cases };
    Ok(Box::from(ASTNodePos::new(start, end, node)))
}

pub fn parse_match_cases(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut cases = vec![];
    it.peek_while_not_token(&Token::Dedent, &mut |it, token_pos| {
        cases.push(*it.parse(&parse_match_case, "match case", token_pos.start)?);
        it.eat_if(&Token::NL);
        Ok(())
    })?;

    it.eat(&Token::Dedent, "match cases")?;
    Ok(cases)
}

fn parse_match_case(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("match case")?;
    let cond = it.parse(&parse_id_maybe_type, "match case", start)?;
    it.eat(&Token::BTo, "match case")?;
    let body = it.parse(&parse_expr_or_stmt, "match case", start)?;

    let node = ASTNode::Case { cond, body };
    Ok(Box::from(ASTNodePos::new(start, body.position.end, node)))
}

pub fn parse_id_maybe_type(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("expression maybe type")?;
    let mutable = it.eat_if(&Token::Mut).is_some();

    let id = it.parse(&parse_expression, "expression maybe type", start)?;
    let _type = it.parse_if(&Token::DoublePoint, &parse_type, "id type", start)?;

    let end = if _type.is_some() { _type.unwrap().position.end } else { id.position.end };
    let node = ASTNode::IdType { id, mutable, _type };
    Ok(Box::from(ASTNodePos::new(start, end, node)))
}
