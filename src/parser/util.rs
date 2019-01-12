use crate::lexer::Token;
use crate::lexer::TokenPos;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn ind_count(it: &mut Peekable<Iter<TokenPos>>) -> i32 {
    let mut ind_count = 0;
    while let Some(&&TokenPos { line: _, pos: _, token: Token::Ind }) = it.peek() {
        next_and!(it, ind_count += 1)
    }

    return ind_count;
}

pub fn detect_double_newline(it: &mut Peekable<Iter<TokenPos>>) -> bool {
    /* empty line */
    if let Some(&&TokenPos { line: _, pos: _, token: Token::NL }) = it.peek() {
        it.next();
        /* followed by newline */
        if let Some(&&TokenPos { line: _, pos: _, token: Token::NL }) = it.peek() {
            it.next();
            /* double empty line */
            if let Some(&&TokenPos { line: _, pos: _, token: Token::NL }) = it.peek() {
                it.next();
                return true;
            }
        }
    }

    return false;
}
