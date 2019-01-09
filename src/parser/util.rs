use crate::lexer::Token;
use crate::lexer::TokenPos;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn ind_count(it: &mut Peekable<Iter<TokenPos>>) -> i32 {
    let mut ind_count = 0;
    while Some(&&TokenPos { line, pos, token: Token::Ind }) == it.peek() {
        next_and!(it, ind_count += 1)
    }

    return ind_count;
}
