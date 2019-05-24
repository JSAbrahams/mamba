use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNodePos;
use crate::parser::parse_result::ParseErr;
use crate::parser::parse_result::ParseErr::Cause;
use crate::parser::parse_result::ParseErr::CustomEOFErr;
use crate::parser::parse_result::ParseErr::EOFErr;
use crate::parser::parse_result::ParseErr::TokenErr;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;

pub struct TPIterator<'a> {
    it: Peekable<Iter<'a, TokenPos>>
}

impl<'a> TPIterator<'a> {
    pub fn new(it: Peekable<Iter<'a, TokenPos>>) -> TPIterator { TPIterator { it } }

    pub fn if_some_eat(&mut self, token: Token) -> ParseResult<()> {
        if self.it.peek().is_some() {
            self.eat(token)
        } else {
            Ok(())
        }
    }

    pub fn eat(&mut self, token: Token) -> ParseResult<()> {
        match self.it.next() {
            Some(TokenPos { token: actual, .. }) if *actual == token => Ok(()),
            Some(&tp) => Err(TokenErr { expected: token, actual: tp.clone() }),
            None => Err(EOFErr { expected: token })
        }
    }

    /// Eat next [Token](crate::lexer::token::Token) if equal to the given
    /// token.
    pub fn eat_if(&mut self, token: Token) -> bool {
        if let Some(TokenPos { token: actual, .. }) = self.it.peek() {
            if *actual == token {
                self.it.next();
                return true;
            }
        }
        false
    }

    pub fn parse(
        &mut self,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult,
        err_msg: &str
    ) -> ParseResult<Box<ASTNodePos>> {
        let current = self.it.peek().cloned();
        match parse_fun(self) {
            Ok(node) => Ok(node),
            Err(err) => Err(Cause {
                parsing:  String::from(err_msg),
                cause:    Box::new(err),
                position: current.cloned()
            })
        }
    }

    pub fn parse_vec(
        &mut self,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult<Vec<ASTNodePos>>,
        err_msg: &str
    ) -> ParseResult<Vec<ASTNodePos>> {
        let current = self.it.peek().cloned();
        match parse_fun(self) {
            Ok(node) => Ok(node),
            Err(err) => Err(Cause {
                parsing:  String::from(err_msg),
                cause:    Box::new(err),
                position: current.cloned()
            })
        }
    }

    pub fn parse_if(
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

    pub fn parse_vec_if(
        &mut self,
        token: Token,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult<Vec<ASTNodePos>>,
        err_msg: &str
    ) -> ParseResult<Vec<ASTNodePos>> {
        match self.it.peek() {
            Some(tp) if tp.token == token => Ok(vec![]),
            _ => Ok(self.parse_vec(parse_fun, err_msg)?)
        }
    }

    pub fn next(
        &mut self,
        match_fun: &Fn(&TokenPos) -> ParseResult,
        none_err: ParseErr
    ) -> ParseResult {
        match self.start_pos() {
            Err(err) => Err(err),
            Ok((st_line, st_pos)) => match self.it.next() {
                Some(token_pos) => match_fun(token_pos),
                None => Err(none_err)
            }
        }
    }

    pub fn peek(
        &mut self,
        match_fun: &Fn(&TokenPos) -> ParseResult,
        none_err: ParseErr
    ) -> ParseResult {
        match self.start_pos() {
            Err(err) => Err(err),
            Ok((st_line, st_pos)) => match self.it.peek() {
                Some(token_pos) => match_fun(token_pos),
                None => Err(none_err)
            }
        }
    }

    pub fn peek_or_none(
        &mut self,
        match_fun: &Fn(&Option<TokenPos>) -> ParseResult
    ) -> ParseResult {
        match self.start_pos() {
            Err(err) => Err(err),
            Ok((st_line, st_pos)) => match_fun(&self.it.peek().cloned().cloned())
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
            Some(TokenPos { st_line, st_pos, .. }) => Ok((*st_line, *st_pos)),
            None => Err(CustomEOFErr { expected: String::from("a token.") })
        }
    }

    pub fn end_pos(&mut self) -> ParseResult<(i32, i32)> {
        match self.it.peek() {
            Some(TokenPos { st_line, st_pos, token }) => Ok((*st_line, *st_pos + token.len())),
            None => Err(CustomEOFErr { expected: String::from("a token.") })
        }
    }
}
