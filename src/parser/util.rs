use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use crate::lexer::Token;

pub fn ind_count(it: &mut Peekable<Iter<Token>>) -> i32 {
    let mut ind_count = 0;
    while Some(&&Token::Ind) == it.peek() { next_and!(it, ind_count += 1) }

    return ind_count;
}
