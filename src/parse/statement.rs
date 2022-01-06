use crate::lex::token::Token;
use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parse::definition::parse_definition;
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::operation::parse_expression;
use crate::parse::result::{custom, expected_one_of};
use crate::parse::result::ParseResult;
use crate::parse::ty::parse_expression_type;

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
                "statement",
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
        "statement",
    )
}

pub fn parse_with(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("with")?;
    it.eat(&Token::With, "with")?;
    let resource = it.parse(&parse_expression, "with", &start)?;

    let alias = it.parse_if(&Token::As, &parse_expression_type, "with id", &start)?;
    let alias = if let Some(alias) = &alias {
        match alias.node.clone() {
            Node::ExpressionType { expr, mutable, ty } => Some((expr, mutable, ty)),
            _ => return Err(custom("Expected expression type", &alias.pos))
        }
    } else {
        None
    };

    it.eat(&Token::Do, "with")?;
    let expr = it.parse(&parse_expr_or_stmt, "with", &start)?;

    let node = Node::With { resource, alias, expr: expr.clone() };
    Ok(Box::from(AST::new(&start.union(&expr.pos), node)))
}

pub fn is_start_statement(tp: &Token) -> bool {
    matches!(tp,  Token::Def
        | Token::Fin
        | Token::Print
        | Token::For
        | Token::While
        | Token::Pass
        | Token::Raise
        | Token::With)
}
