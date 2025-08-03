use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::class::{parse_class, parse_type_def};
use crate::parse::collection::{parse_list_partial, parse_set_or_dict_partial};
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::expression::is_start_expression;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::{Lex, Token};
use crate::parse::result::{custom, ParseResult};
use crate::parse::statement::{is_start_statement, parse_import};

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
            if it.peek_if(&is_start_statement) || it.peek_if(&is_start_expression) {
                let pos = it.peek_next().map_or(start, |next| next.pos);
                return Err(Box::new(custom("statement or expression cannot be immediately followed by another statement or expression", pos)))
            }
            Ok(())
        }
    })?;

    Ok(statements)
}

/// Similar to parse code block, but used in situations where a list is also allowed (when parsing an expression).
pub fn parse_code_block_or_list(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("block block or list")?;
    it.eat_while(&Token::NL);

    it.eat(&Token::LSBrack, "block block or list")?;
    let statements = it.parse_vec(&parse_statements, "block block", start)?;
    let end = statements.last().cloned().map_or(start, |stmt| stmt.pos);

    if statements.len() == 1 && it.peek_if(&|lex: &Lex| lex.token == Token::Comma) {
        parse_list_partial(start, statements.first().unwrap(), it)
    } else {
        it.eat(&Token::RSBrack, "block block or list")?;
        Ok(Box::from(AST::new(
            start.union(end),
            Node::Block { statements },
        )))
    }
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

pub fn parse_code_set_or_set(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("block set or set")?;
    it.eat_while(&Token::NL);

    it.eat(&Token::LCBrack, "block set or set")?;
    let statements = it.parse_vec(&parse_statements, "block set", start)?;
    let end = statements.last().cloned().map_or(start, |stmt| stmt.pos);

    if statements.len() == 1 && it.peek_if(&|lex: &Lex| lex.token == Token::Comma) {
        parse_set_or_dict_partial(start, statements.first().unwrap(), it)
    } else {
        it.eat(&Token::RCBrack, "block set or set")?;
        Ok(Box::from(AST::new(
            start.union(end),
            Node::Block { statements },
        )))
    }
}
