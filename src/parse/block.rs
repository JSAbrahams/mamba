use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::class::{parse_class, parse_type_def};
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::result::{expected_one_of, ParseResult};
use crate::parse::statement::parse_import;

pub fn parse_statements(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("statements")?;
    let mut statements: Vec<AST> = Vec::new();

    it.peek_while_not_tokens(
        &[Token::Dedent, Token::Eof],
        &mut |it, lex| match &lex.token {
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
            Token::Comment(comment) => {
                let end = it.eat(&Token::Comment(comment.clone()), "statements")?;
                let node = Node::Comment { comment: comment.clone() };
                statements.push(AST::new(lex.pos.union(end), node));

                let last_pos = &it.last_pos();
                it.eat_if_not_empty(&Token::NL, "statements", last_pos)?;
                Ok(())
            }
            Token::DocStr(doc_str) => {
                let end = it.eat(&Token::DocStr(doc_str.clone()), "statements")?;
                let node = Node::DocStr { lit: doc_str.clone() };
                statements.push(AST::new(lex.pos.union(end), node));

                let last_pos = &it.last_pos();
                it.eat_if_not_empty(&Token::NL, "statements", last_pos)?;
                Ok(())
            }
            _ => {
                statements.push(*it.parse(&parse_expr_or_stmt, "statements", start)?);
                if it.peek_if(&|lex| lex.token != Token::NL && lex.token != Token::Dedent && lex.token != Token::Eof) {
                    Err(expected_one_of(&[Token::NL, Token::Dedent, Token::Eof], lex, "end of statement"))
                } else {
                    Ok(())
                }
            }
        },
    )?;

    Ok(statements)
}

/// Parse block, and consumes any newlines preceding it.
pub fn parse_block(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("block")?;
    it.eat_while(&Token::NL);

    it.eat(&Token::Indent, "block")?;
    let statements = it.parse_vec(&parse_statements, "block", start)?;
    let end = statements.last().cloned().map_or(start, |stmt| stmt.pos);

    it.eat(&Token::Dedent, "block")?;
    Ok(Box::from(AST::new(start.union(end), Node::Block { statements })))
}
