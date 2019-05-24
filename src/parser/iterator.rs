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

    pub fn peek_is(&mut self, token: Token) -> bool {
        if let Some(tp) = self.it.peek() {
            tp.token == token
        } else {
            false
        }
    }

    pub fn if_next(&mut self, fun: &Fn(&TokenPos) -> bool) -> bool {
        if let Some(tp) = self.it.peek() {
            fun(tp)
        } else {
            false
        }
    }

    pub fn eat(&mut self, token: Token) -> ParseResult<()> {
        match self.it.next() {
            Some(TokenPos { token: actual, .. }) if *actual == token => Ok(()),
            Some(&tp) => Err(TokenErr { expected: token, actual: tp.clone() }),
            None => Err(EOFErr { expected: token })
        }
    }

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
            Some(tp) if tp.token == token => Ok(Some(Box::from(self.parse(parse_fun, err_msg)?))),
            _ => Ok(None)
        }
    }

    pub fn parse_vec_if(
        &mut self,
        token: Token,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult<Vec<ASTNodePos>>,
        err_msg: &str
    ) -> ParseResult<Vec<ASTNodePos>> {
        match self.it.peek() {
            Some(tp) if tp.token == token => Ok(self.parse_vec(parse_fun, err_msg)?),
            _ => Ok(vec![])
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

    pub fn peek_or(
        &mut self,
        match_fun: &Fn(&TokenPos) -> ParseResult,
        default: ParseResult
    ) -> ParseResult {
        match self.start_pos() {
            Err(err) => Err(err),
            Ok((st_line, st_pos)) => match self.it.peek() {
                Some(token_pos) => match_fun(token_pos),
                None => default
            }
        }
    }

    pub fn peek_vec_or(
        &mut self,
        match_fun: &Fn(&TokenPos) -> ParseResult<Vec<ASTNodePos>>,
        default: ParseResult<Vec<ASTNodePos>>
    ) -> ParseResult<Vec<ASTNodePos>> {
        match self.start_pos() {
            Err(err) => Err(err),
            Ok((st_line, st_pos)) => match self.it.peek() {
                Some(token_pos) => match_fun(token_pos),
                None => default
            }
        }
    }

    pub fn peek_if_or_none(
        &mut self,
        token: Token,
        match_fun: &Fn(&Option<TokenPos>) -> ParseResult
    ) -> Option<ParseResult> {
        if self.it.peek().is_some() && self.it.peek().unwrap().token == token {
            Some(match self.start_pos() {
                Err(err) => Err(err),
                Ok((st_line, st_pos)) => match_fun(&self.it.peek().cloned().cloned())
            })
        } else {
            None
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

    pub fn while_some_and(
        &mut self,
        token: Token,
        loop_fn: &Fn(&TokenPos) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.while_some_and_not_fn(&|token_pos| token_pos.token == token, loop_fn)
    }

    pub fn while_some_and_not(
        &mut self,
        token: Token,
        loop_fn: &Fn(&TokenPos) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.while_some_and_not_fn(&|token_pos| token_pos.token != token, loop_fn)
    }

    pub fn while_some_and_not_fn(
        &mut self,
        check_fn: &Fn(&TokenPos) -> bool,
        loop_fn: &Fn(&TokenPos) -> ParseResult<()>
    ) -> ParseResult<()> {
        while let Some(&token_pos) = self.it.peek() {
            if !check_fn(token_pos) {
                break;
            }

            loop_fn(token_pos)?;
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
