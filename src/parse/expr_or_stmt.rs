use crate::lex::token::Token;
use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::block::parse_block;
use crate::parse::control_flow_expr::parse_match_cases;
use crate::parse::iterator::LexIterator;
use crate::parse::operation::parse_expression;
use crate::parse::result::ParseResult;
use crate::parse::statement::is_start_statement;
use crate::parse::statement::parse_statement;
use crate::parse::ty::parse_generics;

pub fn parse_expr_or_stmt(it: &mut LexIterator) -> ParseResult {
    let result = it.peek_or_err(
        &|it, lex| match &lex.token {
            Token::NL => {
                it.eat(&Token::NL, "expression or statement")?;
                it.parse(&parse_block, "expression or statement", &lex.pos)
            }
            token =>
                if is_start_statement(token) {
                    parse_statement(it)
                } else {
                    parse_expression(it)
                },
        },
        &[],
        "expression or statement"
    )?;

    it.peek(
        &|it, lex| match lex.token {
            Token::Raises => parse_raise(*result.clone(), it),
            Token::Handle => parse_handle(*result.clone(), it),
            _ => Ok(result.clone())
        },
        Ok(result.clone())
    )
}

pub fn parse_raise(expr_or_stmt: AST, it: &mut LexIterator) -> ParseResult {
    let start = &it.start_pos("raise")?;
    it.eat(&Token::Raises, "raise")?;

    it.eat(&Token::LSBrack, "raise")?;
    let errors = it.parse_vec(&parse_generics, "raise", start)?;
    it.eat(&Token::RSBrack, "raise")?;
    it.eat_if(&Token::RSBrack);
    let end = errors.last().map_or(start, |stmt| &stmt.pos);

    let node = Node::Raises { expr_or_stmt: Box::from(expr_or_stmt), errors: errors.clone() };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_handle(expr_or_stmt: AST, it: &mut LexIterator) -> ParseResult {
    let start = &it.start_pos("handle")?;
    it.eat(&Token::Handle, "handle")?;
    it.eat(&Token::NL, "handle")?;

    let cases = it.parse_vec(&parse_match_cases, "handle", start)?;
    let end = cases.last().map_or(start, |stmt| &stmt.pos);

    let node = Node::Handle { expr_or_stmt: Box::from(expr_or_stmt), cases: cases.clone() };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}
