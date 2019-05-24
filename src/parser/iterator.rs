use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;

pub struct TPIterator<'a> {
    it: Peekable<Iter<'a, TokenPos>>
}

impl<'a> TPIterator<'a> {
    pub fn new(it: Peekable<Iter<'a, TokenPos>>) -> TPIterator { TPIterator { it } }

    pub fn eat(&mut self, token: Token) -> ParseResult<()> {
        match self.it.next() {
            Some(TokenPos { token: actual, .. }) if *actual == token => Ok(()),
            Some(&tp) => Err(TokenErr { expected: token, actual: tp.clone() }),
            None => Err(EOFErr { expected: token })
        }
    }

    /// Eat next [Token](crate::lexer::token::Token) if equal to the given
    /// token. Never errors.
    pub fn eat_if(&mut self, token: Token) -> ParseResult<()> {
        if let Some(TokenPos { token: actual, .. }) = self.it.peek() {
            if *actual == token {
                self.it.next();
            }
        };
        Ok(())
    }

    pub fn parse(
        &mut self,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult,
        err_msg: &str
    ) -> ParseResult {
        let current = self.it.peek().cloned();
        match parse_fun(self) {
            Ok(node) => Ok(node),
            Err(err) => Err(ParseErr {
                parsing:  String::from(err_msg),
                cause:    Box::new(err),
                position: current.cloned()
            })
        }
    }

    pub fn parse_or_none(
        &mut self,
        token: Token,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult,
        err_msg: &str
    ) -> ParseResult<Option<Box<ASTNodePos>>> {
        match self.it.peek() {
            Some(tp) if tp.token == token => Ok(None),
            _ => Ok(Some(Box::from(self.parse(parse_fun, err_msg)?)))
        }
    }

    pub fn next(
        &mut self,
        match_fun: &Fn(Option<&TokenPos>) -> ParseResult<ASTNode>
    ) -> ParseResult {
        match self.start_pos() {
            Ok((st_line, st_pos)) => {
                let node = match_fun(self.it.next())?;
                let (en_line, en_pos) = self.end_pos()?;
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            Err(err) => Err(err)
        }
    }

    pub fn while_some_and_not(
        &mut self,
        token: Token,
        loop_fn: &Fn() -> ParseResult<()>
    ) -> ParseResult<()> {
        while let Some(&token_pos) = self.it.peek() {
            if token_pos.token == token {
                break;
            }
            loop_fn()?;
        }
        Ok(())
    }

    pub fn start_pos(&mut self) -> ParseResult<(i32, i32)> {
        match self.it.peek() {
            Some(TokenPos { line, pos, .. }) => Ok((*line, *pos)),
            None => Err(CustomEOFErr { expected: String::from("a token.") })
        }
    }

    pub fn end_pos(&mut self) -> ParseResult<(i32, i32)> {
        match self.it.peek() {
            Some(TokenPos { line, pos, token }) => Ok((*line, *pos + token.len())),
            None => Err(CustomEOFErr { expected: String::from("a token.") })
        }
    }
}
