use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::class::{parse_class, parse_type_def};
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::result::{expected_one_of, ParseResult};
use crate::parse::statement::parse_import;

pub fn parse_statements(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("statements")?;
    let mut statements: Vec<AST> = Vec::new();

    it.peek_while_not_tokens(&[], &mut |it, lex| match &lex.token {
        Token::NL => it.eat(&Token::NL, "statements").map(|_| ()),

        Token::Import | Token::From => {
            statements.push(*it.parse(&parse_import, "file", start)?);
            Ok(())
        }
        Token::Type => {
            statements.push(*it.parse(&parse_type_def, "file", start)?);
            Ok(())
        }
        Token::Class => {
            statements.push(*it.parse(&parse_class, "file", start)?);
            Ok(())
        }
        Token::DocStr(doc_str) => {
            let end = it.eat(&Token::DocStr(doc_str.clone()), "statements")?;
            let node = Node::DocStr {
                lit: doc_str.clone(),
            };
            statements.push(AST::new(lex.pos.union(end), node));
            Ok(())
        }
        _ => {
            statements.push(*it.parse(&parse_expr_or_stmt, "statements", start)?);
            if it.peek_if(&|lex| lex.token != Token::NL) {
                Err(Box::from(expected_one_of(
                    &[Token::NL],
                    lex,
                    "end of statement",
                )))
            } else {
                Ok(())
            }
        }
    })?;

    Ok(statements)
}

/// Parse block, and consumes any newlines preceding it.
pub fn parse_code_block(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("block block")?;
    it.eat_while(&Token::NL);

    it.eat(&Token::LSBrack, "block block")?;
    let statements = it.parse_vec(&parse_statements, "block block", start)?;
    let end = statements.last().cloned().map_or(start, |stmt| stmt.pos);

    it.eat(&Token::RSBrack, "block block")?;
    Ok(Box::from(AST::new(
        start.union(end),
        Node::Block { statements },
    )))
}

pub fn parse_code_set(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("block set")?;
    it.eat_while(&Token::NL);

    it.eat(&Token::LCBrack, "block set")?;
    let statements = it.parse_vec(&parse_statements, "block set", start)?;
    let end = statements.last().cloned().map_or(start, |stmt| stmt.pos);

    it.eat(&Token::RCBrack, "block set")?;
    Ok(Box::from(AST::new(
        start.union(end),
        Node::Block { statements },
    )))
}
