use crate::lexer::Token;
use crate::lexer::TokenPos;
use std::error;
use std::fmt;

pub type ParseResult<T> = std::result::Result<T, ParseErr>;

#[derive(Debug)]
pub enum ParseErr {
    UtilBodyErr,
    TokenErr { expected: Token, actual: TokenPos },
    EOFErr { expected: Token },
    IndErr { expected: i32, actual: i32 },
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErr::UtilBodyErr => write!(f, "Util module cannot have a body."),
            ParseErr::EOFErr { expected } => write!(f, "Expected <{}>, but end of file reached.",
                                                    expected),
            ParseErr::TokenErr { expected, actual } =>
                write!(f, "Expected {} at line {}, position {}, but was: {}.",
                       expected,
                       actual.line, actual.pos, actual.token),
            ParseErr::IndErr { expected, actual } =>
                write!(f, "Expected indentation of {}, but was: {}.", expected, actual)
        }
    }
}

impl error::Error for ParseErr {
    fn description(&self) -> &str {
        match self {
            ParseErr::UtilBodyErr => "Util module cannot have a body.",
            ParseErr::EOFErr { expected: _ } => "Expected token but end of file.",
            ParseErr::TokenErr { expected: _, actual: _ } => "Unexpected token encountered.",
            ParseErr::IndErr { expected: _, actual: _ } => "Unexpected indentation."
        }
    }

    fn source(&self) -> Option<&(error::Error + 'static)> { None }
}
