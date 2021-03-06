use crate::lex::token::Token;
use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::operation::parse_expression;
use crate::parse::result::expected_one_of;
use crate::parse::result::ParseResult;
use crate::parse::ty::parse_type;

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
    let cond = it.parse(&parse_expression, "if expression", &start)?;
    it.eat(&Token::Then, "if expression")?;
    let then = it.parse(&parse_expr_or_stmt, "if expression", &start)?;
    let el = it.parse_if(&Token::Else, &parse_expr_or_stmt, "if else branch", &start)?;

    let pos = if let Some(el) = &el { start.union(&el.pos) } else { start.union(&then.pos) };
    let node = Node::IfElse { cond, then, el };

    Ok(Box::from(AST::new(&pos, node)))
}

fn parse_match(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("match")?;
    it.eat(&Token::Match, "match")?;
    let cond = it.parse(&parse_expression, "match", &start)?;
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
    let cond = it.parse(&parse_expression_maybe_type, "match case", &start)?;
    it.eat(&Token::BTo, "match case")?;
    let body = it.parse(&parse_expr_or_stmt, "match case", &start)?;

    let node = Node::Case { cond, body: body.clone() };
    Ok(Box::from(AST::new(&start.union(&body.pos), node)))
}

fn parse_expression_maybe_type(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("expression maybe type")?;
    let mutable = it.eat_if(&Token::Mut).is_some();

    let expr = it.parse(&parse_expression, "expression maybe type", &start)?;
    let ty = it.parse_if(&Token::DoublePoint, &parse_type, "expression maybe type", &start)?;
    let end = ty.clone().map_or(expr.pos.clone(), |t| t.pos);

    let node = Node::ExpressionType { expr, mutable, ty };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}
