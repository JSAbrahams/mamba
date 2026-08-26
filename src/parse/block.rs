use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::class::{parse_class, parse_trait_def};
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::result::expected_one_of;
use crate::parse::result::ParseResult;
use crate::parse::statement::parse_import;

pub fn parse_statements(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("statements")?;
    let mut statements: Vec<AST> = Vec::new();

    it.peek_while_not_tokens(&[Token::End], &mut |it, lex| match &lex.token {
        Token::NL => it.eat(&Token::NL, "statements").map(|_| ()),

        Token::Import | Token::From => {
            statements.push(*it.parse(&parse_import, "file", start)?);
            Ok(())
        }
        Token::Trait => {
            statements.push(*it.parse(&parse_trait_def, "file", start)?);
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
            let ast = it.parse(&parse_expr_or_stmt, "statements", start)?;
            statements.push(*ast.clone());

            if it.peek_if(&|lex| lex.token != Token::NL && lex.token != Token::End) {
                Err(Box::from(expected_one_of(
                    &[Token::NL],
                    lex,
                    &format!("end of statement '{}'", ast.node),
                )))
            } else {
                Ok(())
            }
        }
    })?;

    Ok(statements)
}

/// Parse block, and consumes any newlines preceding it.
pub fn parse_block(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("code block")?;
    it.eat(&Token::Do, "code block")?;

    let statements = it.parse_vec(&parse_statements, "code block", start)?;
    let end = statements.last().cloned().map_or(start, |stmt| stmt.pos);

    it.eat(&Token::End, "code block")?;
    Ok(Box::from(AST::new(
        start.union(end),
        Node::Block { statements },
    )))
}

/// Parse block, and consumes any newlines preceding it.
pub fn parse_set(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("code block")?;
    it.eat(&Token::Where, "code set")?;

    let statements = it.parse_vec(&parse_statements, "code set", start)?;
    let end = statements.last().cloned().map_or(start, |stmt| stmt.pos);

    it.eat(&Token::End, "code set")?;
    Ok(Box::from(AST::new(
        start.union(end),
        Node::Block { statements },
    )))
}
