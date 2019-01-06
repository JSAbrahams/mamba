use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;
use super::super::lexer::Token;

pub fn check_ind(it: &mut Peekable<Iter<Token>>, ind: i32) -> Result<(), String> {
    for i in 0..ind {
        if it.next() != Some(&Token::Ind) {
            return Err(format!("Expected indentation level of {}, but was {}.", ind, i));
        }
    }
    Ok(())
}
