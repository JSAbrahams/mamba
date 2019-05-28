use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNodePos;
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

    pub fn eat(&mut self, token: Token, err_msg: &str) -> ParseResult<(i32, i32)> {
        match self.it.next() {
            Some(TokenPos { token: actual, st_line, st_pos })
                if Token::same_type(actual.clone(), token.clone()) =>
                Ok((*st_line, *st_pos + actual.clone().width())),
            Some(tp) => Err(TokenErr {
                expected: token.clone(),
                actual:   tp.clone(),
                message:  String::from(err_msg)
            }),
            None => Err(EOFErr { expected: token.clone() })
        }
    }

    pub fn eat_if(&mut self, token: Token) -> bool {
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

    pub fn parse_if(
        &mut self,
        token: Token,
        parse_fun: &Fn(&mut TPIterator) -> ParseResult,
        err_msg: &str
    ) -> ParseResult<Option<Box<ASTNodePos>>> {
        match self.it.peek() {
            Some(tp) if Token::same_type(tp.token.clone(), token.clone()) => {
                self.eat(token, err_msg)?;
                Ok(Some(self.parse(parse_fun, err_msg)?))
            }
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
            Some(tp) if Token::same_type(tp.token.clone(), token.clone()) => {
                self.eat(token, err_msg)?;
                Ok(self.parse_vec(parse_fun, err_msg)?)
            }
            _ => Ok(vec![])
        }
    }

    pub fn peek_or_err(
        &mut self,
        match_fun: &Fn(&mut TPIterator, &TokenPos) -> ParseResult,
        err_msg: &str
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => Err(CustomEOFErr { expected: String::from(err_msg) }),
            Some(token_pos) => match_fun(self, token_pos)
        }
    }

    pub fn peek(
        &mut self,
        match_fun: &Fn(&mut TPIterator, &TokenPos) -> ParseResult,
        default: ParseResult,
        err_msg: &str
    ) -> ParseResult {
        match self.it.peek().cloned() {
            None => default,
            Some(token_pos) => match match_fun(self, &token_pos.clone()) {
                Ok(ok) => Ok(ok),
                Err(err) => Err(Cause {
                    parsing:  String::from(err_msg),
                    cause:    Box::new(err),
                    position: Some(token_pos.clone())
                })
            }
        }
    }

    pub fn peek_while_not_tokens(
        &mut self,
        tokens: &[Token],
        loop_fn: &mut FnMut(&mut TPIterator, &TokenPos, i32) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.peek_while_fn(
            &|token_pos| {
                tokens
                    .to_vec()
                    .into_iter()
                    .all(|token| !Token::same_type(token_pos.token.clone(), token.clone()))
            },
            loop_fn
        )
    }

    pub fn peek_while_not_token(
        &mut self,
        token: Token,
        loop_fn: &mut FnMut(&mut TPIterator, &TokenPos, i32) -> ParseResult<()>
    ) -> ParseResult<()> {
        self.peek_while_fn(
            &|token_pos| !Token::same_type(token_pos.token.clone(), token.clone()),
            loop_fn
        )
    }

    pub fn peek_while_fn(
        &mut self,
        check_fn: &Fn(&TokenPos) -> bool,
        loop_fn: &mut FnMut(&mut TPIterator, &TokenPos, i32) -> ParseResult<()>
    ) -> ParseResult<()> {
        let mut no = 1;
        while let Some(&token_pos) = self.it.peek() {
            if !check_fn(token_pos) {
                break;
            }

            loop_fn(self, token_pos, no)?;
            no += 1;
        }
        Ok(())
    }

    pub fn start_pos(&mut self) -> ParseResult<(i32, i32)> {
        match self.it.peek() {
            Some(TokenPos { st_line, st_pos, .. }) => Ok((*st_line, *st_pos)),
            None => Err(CustomEOFErr { expected: String::from("token.") })
        }
    }
}
