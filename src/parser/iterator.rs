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

    pub fn peak_if_fn(&mut self, fun: &Fn(&TokenPos) -> bool) -> bool {
        if let Some(tp) = self.it.peek() {
            fun(tp)
        } else {
            false
        }
    }

    pub fn eat_token(&mut self, token: Token) -> ParseResult<()> {
        match self.it.next() {
            Some(TokenPos { token: actual, .. })
                if Token::same_type(actual.clone(), token.clone()) =>
                Ok(()),
            Some(tp) => Err(TokenErr { expected: token.clone(), actual: tp.clone() }),
            None => Err(EOFErr { expected: token.clone() })
        }
    }

    pub fn eat_if_token(&mut self, token: Token) -> bool {
        if let Some(TokenPos { token: actual, .. }) = self.it.peek() {
            if Token::same_type(actual.clone(), token) {
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

    pub fn parse_if_token(
        &mut self,
        token: Token,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult,
        err_msg: &str
    ) -> ParseResult<Option<Box<ASTNodePos>>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(tp.token.clone(), token) =>
                Ok(Some(Box::from(self.parse(parse_fun, err_msg)?))),
            _ => Ok(None)
        }
    }

    pub fn parse_vec_if_token(
        &mut self,
        token: Token,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult<Vec<ASTNodePos>>,
        err_msg: &str
    ) -> ParseResult<Vec<ASTNodePos>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(tp.token.clone(), token) =>
                Ok(self.parse_vec(parse_fun, err_msg)?),
            _ => Ok(vec![])
        }
    }

    pub fn next_or_err(
        &mut self,
        match_fun: &Fn(&mut TPIterator, &TokenPos) -> ParseResult,
        none_err: ParseErr
    ) -> ParseResult {
        match self.start_pos() {
            Err(err) => Err(err),
            Ok(_) => match self.it.next() {
                Some(token_pos) => match_fun(self, token_pos),
                None => Err(none_err)
            }
        }
    }

    pub fn peek_or_err(
        &mut self,
        match_fun: &Fn(&mut TPIterator, &TokenPos) -> ParseResult,
        none_err: ParseErr
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => Err(none_err),
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
            Some(token_pos) => match_fun(self, token_pos)
        }
    }

    pub fn while_token(
        &mut self,
        token: Token,
        loop_fn: &mut FnMut(&mut TPIterator, &TokenPos) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.while_fn(
            &|token_pos| Token::same_type(token_pos.token.clone(), token.clone()),
            loop_fn
        )
    }

    pub fn while_not_token(
        &mut self,
        token: Token,
        loop_fn: &mut FnMut(&mut TPIterator, &TokenPos) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.while_fn(
            &|token_pos| !Token::same_type(token_pos.token.clone(), token.clone()),
            loop_fn
        )
    }

    pub fn while_fn(
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

    pub fn start_pos(&mut self) -> ParseResult<(i32, i32)> {
        match self.it.peek() {
            Some(TokenPos { st_line, st_pos, .. }) => Ok((*st_line, *st_pos)),
            None => Err(CustomEOFErr { expected: String::from("a token.") })
        }
    }

    pub fn end_pos(&mut self) -> ParseResult<(i32, i32)> {
        match self.it.peek() {
            Some(TokenPos { st_line, st_pos, token }) =>
                Ok((*st_line, *st_pos + token.clone().len())),
            None => Err(CustomEOFErr { expected: String::from("a token.") })
        }
    }
}
