use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::util::count_and_skip_ind;
use std::env;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_block(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> ParseResult<ASTNode> {
    print_parse!(it, ind, "do block");

    let mut nodes = Vec::new();
    loop {
        if it.peek().is_none() { break; }
        if let Some(TokenPos { line: _, pos: _, token: Token::NL }) = it.peek() {
            it.next();

            let next_line_ind = it.clone().take_while(|x| x.token == Token::Ind).count() as i32;
            let token_after_ind = it.clone().skip(next_line_ind as usize).next();

            /* empty line is ignored */
            if token_after_ind.is_some() && token_after_ind.unwrap().token == Token::NL ||
                token_after_ind.is_none() {
                it.next();
                continue;
            } else if next_line_ind < ind {
                break; /* indentation decrease marks end block */
            }
        }

        let this_ind = count_and_skip_ind(it);
        if this_ind != ind {
            let position = it.peek().cloned().cloned();
            return Err(IndErr { actual: this_ind, expected: ind, position });
        }

        let (ast_node, _) = get_or_err_direct!(it, ind, parse_expr_or_stmt, "block");
        nodes.push(ast_node);
    }

    println!("exit do block");
    return Ok((ASTNode::Block(nodes), ind - 1));
}
