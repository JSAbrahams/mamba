use crate::lexer::Token;
use crate::lexer::TokenPos;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use std::env;

pub fn count_and_skip_ind(it: &mut Peekable<Iter<TokenPos>>) -> i32 {
    let mut ind_count = 0;
    while let Some(TokenPos { line: _, pos: _, token: Token::Ind }) = it.peek() {
        next_and!(it, ind_count += 1)
    }

    return ind_count;
}
