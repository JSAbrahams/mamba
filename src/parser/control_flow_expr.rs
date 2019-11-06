use crate::lexer::token::Token;
use crate::parser::_type::parse_type;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::LexIterator;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_cntrl_flow_expr(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match lex.token {
            Token::If => parse_if(it),
            Token::Match => parse_match(it),
            _ => Err(expected_one_of(&[Token::If, Token::Match], lex, "control flow expression"))
        },
        &[Token::If, Token::Match],
        "control flow expression"
    )
}

fn parse_if(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("if expression")?;
    it.eat(&Token::If, "if expressions")?;
    let cond = it.parse(&parse_operation, "if expression", &start)?;
    it.eat(&Token::Then, "if expression")?;
    let then = it.parse(&parse_expr_or_stmt, "if expression", &start)?;
    let _else = it.parse_if(&Token::Else, &parse_expr_or_stmt, "if else branch", &start)?;

    let pos =
        if let Some(_else) = &_else { start.union(&_else.pos) } else { start.union(&then.pos) };
    let node = Node::IfElse { cond, then: then.clone(), _else };

    Ok(Box::from(AST::new(&pos, node)))
}

fn parse_match(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("match")?;
    it.eat(&Token::Match, "match")?;
    let cond = it.parse(&parse_operation, "match", &start)?;
    it.eat(&Token::NL, "match")?;
    let cases = it.parse_vec(&parse_match_cases, "match", &start)?;
    let end = cases.last().cloned().map_or(cond.pos.clone(), |case| case.pos);

    let node = Node::Match { cond, cases };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_match_cases(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.eat(&Token::Indent, "match cases")?;
    let mut cases = vec![];
    it.peek_while_not_token(&Token::Dedent, &mut |it, _| {
        cases.push(*it.parse(&parse_match_case, "match case", &start)?);
        it.eat_if(&Token::NL);
        Ok(())
    })?;

    it.eat(&Token::Dedent, "match cases")?;
    Ok(cases)
}

fn parse_match_case(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("match case")?;
    let cond = it.parse(&parse_id_maybe_type, "match case", &start)?;
    it.eat(&Token::BTo, "match case")?;
    let body = it.parse(&parse_expr_or_stmt, "match case", &start)?;

    let node = Node::Case { cond, body: body.clone() };
    Ok(Box::from(AST::new(&start.union(&body.pos), node)))
}

pub fn parse_id_maybe_type(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("expression maybe type")?;
    let mutable = it.eat_if(&Token::Mut).is_some();

    let id = it.parse(&parse_expression, "expression maybe type", &start)?;
    let _type = it.parse_if(&Token::DoublePoint, &parse_type, "id type", &start)?;

    let end = _type.clone().map_or(id.pos.clone(), |t| t.pos);
    let node = Node::IdType { id, mutable, _type };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}
