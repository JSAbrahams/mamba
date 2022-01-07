use std::iter::Peekable;
use std::slice::Iter;

use itertools::multipeek;

use crate::common::position::Position;
use crate::lex::token::Lex;
use crate::lex::token::Token;
use crate::parse::ast::AST;
use crate::parse::result::eof_expected_one_of;
use crate::parse::result::expected;
use crate::parse::result::ParseResult;

pub struct LexIterator<'a> {
    it: Peekable<Iter<'a, Lex>>,
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

    pub fn peek_if_followed_by(&mut self, token: &Token, final_token: &Token) -> bool {
        if self.it.peek().map(|l| l.token.clone()) != Some(token.clone()) {
            return false;
        }

        let mut multi_peek = multipeek(self.it.clone());
        let mut last_token = None;
        while let Some(lex) = multi_peek.peek() {
            let peeked_token = lex.token.clone();
            last_token = Some(peeked_token.clone());

            if last_token == Some(token.clone()) && peeked_token == final_token.clone() {
                return true;
            } else if peeked_token != token.clone() { break; }
        }

        last_token == Some(final_token.clone())
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

    /// Eat given token until another token is encountered.
    /// Gives position of last consumed token.
    pub fn eat_while(&mut self, token: &Token) -> Option<Position> {
        let mut last_pos = None;
        while self.it.peek().map(|l| l.token.clone()) == Some(token.clone()) {
            last_pos = self.eat_if(token);
        }

        last_pos
    }

    pub fn eat_if_not_empty(&mut self, token: &Token, err_msg: &str, last_pos: &Option<Position>)
                            -> ParseResult<Option<Position>> {
        if self.it.peek().is_some() {
            self.eat(token, err_msg)
                .map(Some)
                .map_err(|err| err.clone_with_cause(err_msg, &last_pos.clone().unwrap_or_default()))
        } else {
            Ok(None)
        }
    }

    pub fn parse(
        &mut self,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult,
        cause: &str,
        start: &Position,
    ) -> ParseResult<Box<AST>> {
        parse_fun(self).map_err(|err| err.clone_with_cause(cause, start))
    }

    pub fn parse_vec(
        &mut self,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult<Vec<AST>>,
        cause: &str,
        start: &Position,
    ) -> ParseResult<Vec<AST>> {
        parse_fun(self).map_err(|err| err.clone_with_cause(cause, start))
    }

    pub fn parse_if(
        &mut self,
        token: &Token,
        parse_fun: &dyn Fn(&mut LexIterator) -> ParseResult,
        err_msg: &str,
        start: &Position,
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
        start: &Position,
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
        eof_err_msg: &str,
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => Err(eof_expected_one_of(eof_expected, eof_err_msg)),
            Some(lex) => match_fun(self, lex)
        }
    }

    pub fn peek(
        &mut self,
        match_fun: &dyn Fn(&mut LexIterator, &Lex) -> ParseResult,
        default: ParseResult,
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => default,
            Some(lex) => match_fun(self, &lex.clone())
        }
    }

    pub fn peek_while_not_tokens(
        &mut self,
        tokens: &[Token],
        loop_fn: &mut dyn FnMut(&mut LexIterator, &Lex) -> ParseResult<()>,
    ) -> ParseResult<()> {
        self.peek_while_fn(
            &|lex| tokens.iter().all(|token| !Token::same_type(&lex.token, &token)),
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
            Some(Lex { pos, .. }) => Ok(pos.clone()),
            None => Err(eof_expected_one_of(&[], &format!("start of a {}", msg)))
        }
    }

    pub fn last_pos(&mut self) -> Option<Position> {
        self.it.peek().map(|lex| lex.pos.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::common::position::CaretPos;

    use super::*;

    #[test]
    fn test_peek_followed_by() {
        let l1 = Lex::new(&CaretPos::default().offset_pos(0), Token::Neq);
        let l2 = Lex::new(&CaretPos::default().offset_pos(1), Token::Neq);
        let l3 = Lex::new(&CaretPos::default().offset_pos(2), Token::Eq);
        let lex = vec![l1, l2, l3];
        let mut it = LexIterator::new(lex.iter().peekable());

        assert!(it.peek_if_followed_by(&Token::Neq, &Token::Eq));
        assert!(it.peek_if_followed_by(&Token::Neq, &Token::Neq));

        assert_eq!(it.peek_if_followed_by(&Token::Neq, &Token::Not), false);
        assert_eq!(it.peek_if_followed_by(&Token::Eq, &Token::Eq), false);
        assert_eq!(it.peek_if_followed_by(&Token::Not, &Token::Not), false);
    }


    #[test]
    fn test_peek_followed_by_leaves_iter_unmodified() {
        let l1 = Lex::new(&CaretPos::default().offset_pos(0), Token::Neq);
        let l2 = Lex::new(&CaretPos::default().offset_pos(1), Token::Eq);
        let lex = vec![l1, l2];
        let mut lex_iter = LexIterator::new(lex.iter().peekable());

        lex_iter.peek_if_followed_by(&Token::Neq, &Token::Eq);
        assert_eq!(lex_iter.it.peek().map(|l| l.token.clone()), Some(Token::Neq));
    }
}