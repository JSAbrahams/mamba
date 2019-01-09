use crate::lexer::Token;
use crate::lexer::TokenPos;
use std::error;
use std::fmt;

pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    TokenError(TokenPos, Token),
    EOFError(Token),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::EOFError(token) =>
                write!(f, "Expected <{}>, but end of file", token),
            ParseError::TokenError(actual, token) =>
                write!(f, "Expected {} at line {} position {}, but was {}.",
                       token,
                       actual.line, actual.pos, actual.token)
        }
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::EOFError(token) =>
                format!("Expected <{}>, but end of file", token).as_ref(),
            ParseError::TokenError(actual, token) =>
                format!("Expected {} at line {} position {}, but was {}.",
                        token,
                        actual.line, actual.pos, actual.token).as_ref()
        }
    }

    fn source(&self) -> Option<&(error::Error + 'static)> { None }
}
