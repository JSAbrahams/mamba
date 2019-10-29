use crate::lexer::token::Lex;
use crate::lexer::token::Token;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::iterator::LexIterator;
use crate::parser::parse_result::expected;
use crate::parser::parse_result::ParseResult;

// TODO look at whether we can handle class and type tokens more elegantly
pub fn parse_statements(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("block")?;
    let mut statements: Vec<AST> = Vec::new();

    it.peek_while_not_tokens(
        &[Token::Dedent, Token::Class, Token::Type],
        &mut |it, lex| match &lex.token {
            Token::NL => {
                it.eat(&Token::NL, "block")?;
                Ok(())
            }
            Token::Comment(comment) => {
                let end = it.eat(&Token::Comment(comment.clone()), "block")?;
                let node = Node::Comment { comment: comment.clone() };
                statements.push(AST::new(&lex.pos.union(&end), node));
                Ok(())
            }
            _ => {
                statements.push(*it.parse(&parse_expr_or_stmt, "block", &start)?);
                Ok(())
            }
        }
    )?;

    Ok(statements)
}

pub fn parse_block(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("block")?;
    it.eat(&Token::Indent, "block")?;
    let statements = it.parse_vec(&parse_statements, "block", &start)?;
    let end = statements.last().cloned().map_or(start.clone(), |stmt| stmt.pos);

    it.eat(&Token::Dedent, "block")?;
    Ok(Box::from(AST::new(&start.union(&end), Node::Block { statements })))
}
