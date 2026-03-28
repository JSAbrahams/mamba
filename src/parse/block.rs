use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::class::{parse_class, parse_type_def};
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::expression::is_start_expression;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::result::{custom, ParseResult};
use crate::parse::statement::{is_start_statement, parse_import};

pub fn parse_statements(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("statements")?;
    let mut statements: Vec<AST> = Vec::new();

    it.peek_while_not_tokens(&[Token::End], &mut |it, lex| match &lex.token {
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
pub fn parse_code_block(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("code block")?;
    it.eat_while(&Token::NL);

    it.eat(&Token::Do, "code block")?;
    let statements = it.parse_vec(&parse_statements, "code block", start)?;

    let end = it.eat(&Token::End, "code block")?;
    Ok(Box::from(AST::new(
        start.union(end),
        Node::Block { statements },
    )))
}

pub fn parse_code_set(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("code set")?;
    it.eat_while(&Token::NL);

    it.eat(&Token::Where, "code set")?;
    let statements = it.parse_vec(&parse_statements, "code set", start)?;
    let end = statements.last().cloned().map_or(start, |stmt| stmt.pos);

    it.eat(&Token::End, "code set")?;
    Ok(Box::from(AST::new(
        start.union(end),
        Node::Block { statements },
    )))
}
