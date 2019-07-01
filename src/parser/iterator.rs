use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNodePos;
use crate::parser::parse_result::eof_expected_one_of;
use crate::parser::parse_result::expected;
use crate::parser::parse_result::ParseResult;
use std::iter::Peekable;
use std::slice::Iter;

pub struct TPIterator<'a> {
    it: Peekable<Iter<'a, TokenPos>>
}

impl<'a> TPIterator<'a> {
    pub fn new(it: Peekable<Iter<'a, TokenPos>>) -> TPIterator { TPIterator { it } }

    pub fn peak_if_fn(&mut self, fun: &Fn(&TokenPos) -> bool) -> bool {
        if let Some(tp) = self.it.peek() {
            fun(tp)
        } else {
            false
        }
    }

    pub fn eat(&mut self, token: &Token, err_msg: &str) -> ParseResult<(i32, i32)> {
        match self.it.next() {
            Some(TokenPos { token: actual, st_line, st_pos })
                if Token::same_type(actual, token) =>
                Ok((*st_line, *st_pos + actual.clone().width())),
            Some(token_pos) => Err(expected(token, token_pos, err_msg)),
            None => Err(eof_expected_one_of(&[token.clone()], err_msg))
        }
    }

    pub fn eat_if(&mut self, token: &Token) -> Option<(i32, i32)> {
        if let Some(TokenPos { token: actual, .. }) = self.it.peek() {
            if Token::same_type(actual, token) {
                return match self.eat(token, "") {
                    Ok(pos) => Some(pos),
                    Err(_) => Some((0, 0))
                };
            }
        }
        None
    }

    pub fn parse(
        &mut self,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult
    ) -> ParseResult<Box<ASTNodePos>> {
        parse_fun(self)
    }

    pub fn parse_vec(
        &mut self,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult<Vec<ASTNodePos>>
    ) -> ParseResult<Vec<ASTNodePos>> {
        parse_fun(self)
    }

    pub fn parse_if(
        &mut self,
        token: &Token,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult,
        err_msg: &str
    ) -> ParseResult<Option<Box<ASTNodePos>>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(&tp.token, token) => {
                self.eat(token, err_msg)?;
                Ok(Some(self.parse(parse_fun)?))
            }
            _ => Ok(None)
        }
    }

    pub fn parse_vec_if(
        &mut self,
        token: &Token,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult<Vec<ASTNodePos>>,
        err_msg: &str
    ) -> ParseResult<Vec<ASTNodePos>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(&tp.token, token) => {
                self.eat(token, err_msg)?;
                Ok(self.parse_vec(parse_fun)?)
            }
            _ => Ok(vec![])
        }
    }

    pub fn peek_or_err(
        &mut self,
        match_fun: &Fn(&mut TPIterator, &TokenPos) -> ParseResult,
        eof_expected: &[Token],
        eof_err_msg: &str
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => Err(eof_expected_one_of(eof_expected, eof_err_msg)),
            Some(token_pos) => match_fun(self, token_pos)
        }
    }

    pub fn peek(
        &mut self,
        match_fun: &Fn(&mut TPIterator, &TokenPos) -> ParseResult,
        default: ParseResult
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => default,
            Some(token_pos) => match_fun(self, &token_pos.clone())
        }
    }

    pub fn peek_while_not_tokens(
        &mut self,
        tokens: &[Token],
        loop_fn: &mut FnMut(&mut TPIterator, &TokenPos) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.peek_while_fn(
            &|token_pos| {
                tokens.to_vec().into_iter().all(|token| !Token::same_type(&token_pos.token, &token))
            },
            loop_fn
        )
    }

    pub fn peek_while_not_token(
        &mut self,
        token: &Token,
        loop_fn: &mut FnMut(&mut TPIterator, &TokenPos) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.peek_while_fn(&|token_pos| !Token::same_type(&token_pos.token, token), loop_fn)
    }

    pub fn peek_while_fn(
        &mut self,
        check_fn: &Fn(&TokenPos) -> bool,
        loop_fn: &mut FnMut(&mut TPIterator, &TokenPos) -> ParseResult<()>
    ) -> ParseResult<()> {
        while let Some(&token_pos) = self.it.peek() {
            if !check_fn(token_pos) {
                break;
            }
            loop_fn(self, token_pos)?;
        }
        Ok(())
    }

    pub fn start_pos(&mut self, msg: &str) -> ParseResult<(i32, i32)> {
        match self.it.peek() {
            Some(TokenPos { st_line, st_pos, .. }) => Ok((*st_line, *st_pos)),
            None => Err(eof_expected_one_of(&[], msg))
        }
    }
}
