use std::iter::Peekable;
use std::slice::Iter;

use crate::common::position::Position;
use crate::common::result::WithCause;
use crate::parse::lex::token::Lex;
use crate::parse::lex::token::Token;
use crate::parse::result::eof_expected_one_of;
use crate::parse::result::expected;
use crate::parse::result::ParseResult;

#[derive(Debug)]
pub struct LexIterator<'a> {
    it: Peekable<Iter<'a, Lex>>,
}

impl<'a> LexIterator<'a> {
    pub fn new(it: Peekable<Iter<'a, Lex>>) -> LexIterator<'a> {
        LexIterator { it }
    }

    pub fn peek_if(&mut self, fun: &dyn Fn(&Lex) -> bool) -> bool {
        if let Some(tp) = self.it.peek() {
            fun(tp)
        } else {
            false
        }
    }

    /// Look past any number of `skip` tokens for a token matching `fun`.
    /// If found, the `skip` tokens (but not the matched token) are consumed.
    /// Otherwise, the iterator is left untouched.
    pub fn peek_if_skipping(&mut self, skip: &Token, fun: &dyn Fn(&Lex) -> bool) -> bool {
        let mut lookahead = self.it.clone();
        while let Some(lex) = lookahead.peek() {
            if Token::same_type(&lex.token, skip) {
                lookahead.next();
            } else {
                break;
            }
        }

        match lookahead.peek() {
            Some(lex) if fun(lex) => {
                self.it = lookahead;
                true
            }
            _ => false,
        }
    }

    pub fn eat(&mut self, token: &Token, err_msg: &str) -> ParseResult<Position> {
        match self.it.next() {
            Some(Lex { token: actual, pos }) if Token::same_type(actual, token) => Ok(*pos),
            Some(lex) => Err(Box::from(expected(token, lex, err_msg))),
            None => Err(Box::from(eof_expected_one_of(
                std::slice::from_ref(token),
                err_msg,
            ))),
        }
    }

    pub fn eat_if(&mut self, token: &Token) -> Option<Position> {
        if let Some(Lex { token: actual, .. }) = self.it.peek() {
            if Token::same_type(actual, token) {
                return self.eat(token, "").ok();
            }
        }
        None
    }

    /// Eat given token until another token is encountered.
    /// Gives position of last consumed token.
    pub fn eat_while(&mut self, token: &Token) -> Option<Position> {
        let mut last_pos = None;
        while self.it.peek().map(|l| l.token.clone()) == Some(token.clone()) {
            last_pos = self.eat_if(token);
        }

        last_pos
    }

    pub fn parse<T>(
        &mut self,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult<T>,
        cause: &str,
        start: Position,
    ) -> ParseResult<T> {
        parse_fun(self).map_err(|err| Box::from(err.with_cause(cause, start)))
    }

    pub fn parse_vec<T>(
        &mut self,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult<Vec<T>>,
        cause: &str,
        start: Position,
    ) -> ParseResult<Vec<T>> {
        parse_fun(self).map_err(|err| Box::from(err.with_cause(cause, start)))
    }

    pub fn parse_if<T>(
        &mut self,
        token: &Token,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult<T>,
        err_msg: &str,
        start: Position,
    ) -> ParseResult<Option<T>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(&tp.token, token) => {
                self.eat(token, err_msg)?;
                Ok(Some(self.parse(parse_fun, err_msg, start)?))
            }
            _ => Ok(None),
        }
    }

    pub fn parse_vec_if<T>(
        &mut self,
        token: &Token,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult<Vec<T>>,
        err_msg: &str,
        start: Position,
    ) -> ParseResult<Vec<T>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(&tp.token, token) => {
                self.eat(token, err_msg)?;
                Ok(self.parse_vec(parse_fun, err_msg, start)?)
            }
            _ => Ok(vec![]),
        }
    }

    pub fn peek_or_err(
        &mut self,
        match_fun: &dyn Fn(&mut LexIterator, &Lex) -> ParseResult,
        eof_expected: &[Token],
        eof_err_msg: &str,
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => Err(Box::from(eof_expected_one_of(eof_expected, eof_err_msg))),
            Some(lex) => match_fun(self, lex),
        }
    }

    pub fn peek(
        &mut self,
        match_fun: &dyn Fn(&mut LexIterator, &Lex) -> ParseResult,
        default: ParseResult,
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => default,
            Some(lex) => match_fun(self, &lex.clone()),
        }
    }

    pub fn peek_next(&mut self) -> Option<Lex> {
        self.it.peek().cloned().cloned()
    }

    pub fn peek_while_not_tokens(
        &mut self,
        tokens: &[Token],
        loop_fn: &mut dyn FnMut(&mut LexIterator, &Lex) -> ParseResult<()>,
    ) -> ParseResult<()> {
        self.peek_while_fn(
            &|lex| {
                tokens
                    .iter()
                    .all(|token| !Token::same_type(&lex.token, token))
            },
            loop_fn,
        )
    }

    pub fn peek_while_not_token(
        &mut self,
        token: &Token,
        loop_fn: &mut dyn FnMut(&mut LexIterator, &Lex) -> ParseResult<()>,
    ) -> ParseResult<()> {
        self.peek_while_fn(&|lex| !Token::same_type(&lex.token, token), loop_fn)
    }

    /// Peek while certain function evaluates to true.
    /// Function always evaluates to false if the next token is [Token::EOF].
    pub fn peek_while_fn(
        &mut self,
        check_fn: &dyn Fn(&Lex) -> bool,
        loop_fn: &mut dyn FnMut(&mut LexIterator, &Lex) -> ParseResult<()>,
    ) -> ParseResult<()> {
        while let Some(&lex) = self.it.peek() {
            if !check_fn(lex) {
                break;
            }
            loop_fn(self, lex)?;
        }
        Ok(())
    }

    pub fn start_pos(&mut self, msg: &str) -> ParseResult<Position> {
        match self.it.peek() {
            Some(Lex { pos, .. }) => Ok(*pos),
            None => Err(Box::from(eof_expected_one_of(
                &[],
                &format!("start of a {msg}"),
            ))),
        }
    }
}
