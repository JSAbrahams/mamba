use crate::lexer::token::Token;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_cntrl_flow_expr(it: &mut TPIterator) -> ParseResult {
    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::If => parse_if(it),
            Token::Match => parse_match(it),
            _ => Err(CustomErr {
                expected: "control flow expression".to_string(),
                actual:   token_pos.clone()
            })
        },
        CustomEOFErr { expected: "control flow expression".to_string() }
    )
}

fn parse_if(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    it.eat_token(Token::If)?;
    let cond = it.parse(&parse_expression, "if condition")?;
    it.eat_token(Token::Then)?;
    let then = it.parse(&parse_expr_or_stmt, "if then branch")?;
    let _else = it.parse_if_token(Token::Else, &parse_expr_or_stmt, "if else branch")?;

    let (en_line, en_pos) = (then.en_line, then.en_pos);
    let node = ASTNode::IfElse { cond, then, _else };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_match(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat_token(Token::Match)?;

    let cond = it.parse(&parse_expression, "match expression")?;
    it.eat_token(Token::NL)?;
    let cases = it.parse_vec(&parse_match_cases, "match cases")?;

    let (en_line, en_pos) = match (&cond, cases.last()) {
        (_, Some(ast_node_pos)) => (ast_node_pos.en_line, ast_node_pos.en_pos),
        (cond, _) => (cond.en_line, cond.en_pos)
    };

    let node = ASTNode::Match { cond, cases };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_match_cases(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat_token(Token::Indent)?;

    let mut cases = Vec::new();
    it.while_not_token(Token::Dedent, &mut |it, _| {
        cases.push(*it.parse(&parse_match_case, "match case")?);
        it.eat_if_token(Token::NL);
        Ok(())
    })?;

    it.eat_token(Token::Dedent)?;
    Ok(cases)
}

fn parse_match_case(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;

    let cond = it.parse(&parse_expression_maybe_type, "match case")?;
    it.eat_token(Token::BTo)?;
    let body = it.parse(&parse_expr_or_stmt, "then")?;

    let (en_line, en_pos) = (body.en_line, body.en_pos);
    let node = ASTNode::Case { cond, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_expression_maybe_type(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let mutable = it.eat_if_token(Token::Mut);

    let id = it.parse(&parse_expression, "id maybe type")?;
    let _type = it.parse_if_token(Token::DoublePoint, &parse_type, "id type")?;

    let (en_line, en_pos) = match &_type {
        Some(_type) => (_type.en_line, _type.en_pos),
        _ => (id.en_line, id.en_pos)
    };
    let node = ASTNode::IdType { id, mutable, _type };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}
