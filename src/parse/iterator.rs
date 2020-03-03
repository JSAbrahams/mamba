use std::iter::Peekable;
use std::slice::Iter;

use crate::common::position::Position;
use crate::lex::token::Lex;
use crate::lex::token::Token;
use crate::parse::ast::AST;
use crate::parse::result::eof_expected_one_of;
use crate::parse::result::expected;
use crate::parse::result::ParseResult;

pub struct LexIterator<'a> {
    it: Peekable<Iter<'a, Lex>>
}

impl<'a> LexIterator<'a> {
    pub fn new(it: Peekable<Iter<'a, Lex>>) -> LexIterator { LexIterator { it } }

    pub fn peek_if(&mut self, fun: &dyn Fn(&Lex) -> bool) -> bool {
        if let Some(tp) = self.it.peek() {
            fun(tp)
        } else {
            false
        }
    }

    pub fn eat(&mut self, token: &Token, err_msg: &str) -> ParseResult<Position> {
        match self.it.next() {
            Some(Lex { token: actual, pos }) if Token::same_type(actual, token) => Ok(pos.clone()),
            Some(lex) => Err(expected(token, lex, err_msg)),
            None => Err(eof_expected_one_of(&[token.clone()], err_msg))
        }
    }

    pub fn eat_if(&mut self, token: &Token) -> Option<Position> {
        if let Some(Lex { token: actual, .. }) = self.it.peek() {
            if Token::same_type(actual, token) {
                return match self.eat(token, "") {
                    Ok(pos) => Some(pos),
                    Err(_) => None
                };
            }
        }
        None
    }

    pub fn parse(
        &mut self,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult,
        cause: &str,
        start: &Position
    ) -> ParseResult<Box<AST>> {
        parse_fun(self).map_err(|err| err.clone_with_cause(cause, start.clone()))
    }

    pub fn parse_vec(
        &mut self,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult<Vec<AST>>,
        cause: &str,
        start: &Position
    ) -> ParseResult<Vec<AST>> {
        parse_fun(self).map_err(|err| err.clone_with_cause(cause, start.clone()))
    }

    pub fn parse_if(
        &mut self,
        token: &Token,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult,
        err_msg: &str,
        start: &Position
    ) -> ParseResult<Option<Box<AST>>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(&tp.token, token) => {
                self.eat(token, err_msg)?;
                Ok(Some(self.parse(parse_fun, err_msg, start)?))
            }
            _ => Ok(None)
        }
    }

    pub fn parse_vec_if(
        &mut self,
        token: &Token,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult<Vec<AST>>,
        err_msg: &str,
        start: &Position
    ) -> ParseResult<Vec<AST>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(&tp.token, token) => {
                self.eat(token, err_msg)?;
                Ok(self.parse_vec(parse_fun, err_msg, start)?)
            }
            _ => Ok(vec![])
        }
    }

    pub fn peek_or_err(
        &mut self,
        match_fun: &dyn Fn(&mut LexIterator, &Lex) -> ParseResult,
        eof_expected: &[Token],
        eof_err_msg: &str
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => Err(eof_expected_one_of(eof_expected, eof_err_msg)),
            Some(lex) => match_fun(self, lex)
        }
    }

    pub fn peek(
        &mut self,
        match_fun: &dyn Fn(&mut LexIterator, &Lex) -> ParseResult,
        default: ParseResult
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => default,
            Some(lex) => match_fun(self, &lex.clone())
        }
    }

    pub fn peek_while_not_tokens(
        &mut self,
        tokens: &[Token],
        loop_fn: &mut dyn FnMut(&mut LexIterator, &Lex) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.peek_while_fn(
            &|lex| tokens.to_vec().into_iter().all(|token| !Token::same_type(&lex.token, &token)),
            loop_fn
        )
    }

    pub fn peek_while_not_token(
        &mut self,
        token: &Token,
        loop_fn: &mut dyn FnMut(&mut LexIterator, &Lex) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.peek_while_fn(&|lex| !Token::same_type(&lex.token, token), loop_fn)
    }

    pub fn peek_while_fn(
        &mut self,
        check_fn: &dyn Fn(&Lex) -> bool,
        loop_fn: &mut dyn FnMut(&mut LexIterator, &Lex) -> ParseResult<()>
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
            Some(Lex { pos, .. }) => Ok(pos.clone()),
            None => Err(eof_expected_one_of(&[], &format!("start of a {}", msg)))
        }
    }
}
