use crate::lexer::token::Token;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parser::definition::parse_definition;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::LexIterator;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_statement(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match lex.token {
            Token::Print => {
                it.eat(&Token::Print, "statement")?;
                let expr = it.parse(&parse_expression, "statement", &lex.pos)?;
                let node = Node::Print { expr: expr.clone() };
                Ok(Box::from(AST::new(&lex.pos.union(&expr.pos), node)))
            }
            Token::Pass => {
                let end = it.eat(&Token::Pass, "statement")?;
                Ok(Box::from(AST::new(&end, Node::Pass)))
            }
            Token::Retry => {
                let end = it.eat(&Token::Retry, "statement")?;
                Ok(Box::from(AST::new(&end, Node::Retry)))
            }
            Token::Raise => {
                it.eat(&Token::Raise, "statement")?;
                let error = it.parse(&parse_expression, "statement", &lex.pos)?;
                let node = Node::Raise { error: error.clone() };
                Ok(Box::from(AST::new(&lex.pos.union(&error.pos), node)))
            }
            Token::Def => parse_definition(it),
            Token::With => parse_with(it),
            Token::For | Token::While => parse_cntrl_flow_stmt(it),
            _ => Err(expected_one_of(
                &[
                    Token::Print,
                    Token::Pass,
                    Token::Raise,
                    Token::Def,
                    Token::With,
                    Token::For,
                    Token::While
                ],
                lex,
                "statement"
            ))
        },
        &[
            Token::Print,
            Token::Pass,
            Token::Raise,
            Token::Def,
            Token::With,
            Token::For,
            Token::While
        ],
        "statement"
    )
}

pub fn parse_with(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("with")?;
    it.eat(&Token::With, "with")?;
    let resource = it.parse(&parse_expression, "with", &start)?;
    let _as = it.parse_if(&Token::As, &parse_id_maybe_type, "with id", &start)?;
    let expr = it.parse(&parse_expr_or_stmt, "with", &start)?;

    let node = Node::With { resource, _as, expr: expr.clone() };
    Ok(Box::from(AST::new(&start.union(&expr.pos), node)))
}

pub fn is_start_statement(tp: &Token) -> bool {
    match tp {
        Token::Def
        | Token::Mut
        | Token::Print
        | Token::For
        | Token::While
        | Token::Retry
        | Token::Pass
        | Token::Raise
        | Token::With => true,
        _ => false
    }
}
