use crate::lexer::token::Token;
use crate::parser::_type::parse_id;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::iterator::LexIterator;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_cntrl_flow_stmt(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match lex.token {
            Token::While => parse_while(it),
            Token::For => parse_for(it),
            Token::Break => {
                let end = it.eat(&Token::Break, "control flow statement")?;
                Ok(Box::from(AST::new(&lex.pos.union(&end), Node::Break)))
            }
            Token::Continue => {
                let end = it.eat(&Token::Continue, "control flow statement")?;
                Ok(Box::from(AST::new(&lex.pos.union(&end), Node::Continue)))
            }
            _ => Err(expected_one_of(
                &[Token::While, Token::For, Token::Break, Token::Continue],
                lex,
                "control flow statement"
            ))
        },
        &[Token::While, Token::For, Token::Break, Token::Continue],
        "control flow statement"
    )
}

fn parse_while(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("while statement")?;
    it.eat(&Token::While, "while statement")?;
    let cond = it.parse(&parse_operation, "while statement", &start)?;
    it.eat(&Token::Do, "while")?;
    let body = it.parse(&parse_expr_or_stmt, "while statement", &start)?;

    let node = Node::While { cond, body: body.clone() };
    Ok(Box::from(AST::new(&start.union(&body.pos), node)))
}

fn parse_for(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("for statement")?;
    it.eat(&Token::For, "for statement")?;
    let expr = it.parse(&parse_id, "for statement", &start)?;
    it.eat(&Token::In, "for statement")?;
    let col = it.parse(&parse_operation, "for statement", &start)?;
    it.eat(&Token::Do, "for statement")?;
    let body = it.parse(&parse_expr_or_stmt, "for statement", &start)?;

    let node = Node::For { expr, col, body: body.clone() };
    Ok(Box::from(AST::new(&start.union(&body.pos), node)))
}
