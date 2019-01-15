use crate::lexer::Token;
use crate::lexer::TokenPos;
use std::error;
use std::fmt;

pub type ParseResult<T> = std::result::Result<(T, i32), ParseErr>;

#[derive(Debug)]
pub enum ParseErr {
    UtilBodyErr,
    ParseErr { parsing: String, cause: Box<ParseErr>, position: Option<TokenPos> },
    CustomErr { expected: String, actual: TokenPos },
    TokenErr { expected: Token, actual: TokenPos },
    EOFErr { expected: Token },
    CustomEOFErr { expected: String },
    IndErr { expected: i32, actual: i32, position: Option<TokenPos> },
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseErr::ParseErr { ref parsing, ref cause, ref position } => match cause.fmt(f) {
                Ok(_) => match position {
                    Some(pos) => write!(f, "\nIn <{}> at {}:{}", parsing, pos.line, pos.pos),
                    None => write!(f, "\nIn <{}>", parsing)
                }
                err => err
            }
            ParseErr::UtilBodyErr => write!(f, "\nUtil module cannot have a body."),
            ParseErr::EOFErr { expected } =>
                write!(f, "\nExpected <{}>, but end of file.", expected),
            ParseErr::CustomErr { expected, actual } =>
                write!(f, "\nExpected <{}> at {}:{} (line:col) but was {}.",
                       expected,
                       actual.line, actual.pos, actual.token),
            ParseErr::TokenErr { expected, actual } =>
                write!(f, "\nExpected {} at {}:{} (line:col) but was {}.",
                       expected,
                       actual.line, actual.pos, actual.token),
            ParseErr::CustomEOFErr { expected } =>
                write!(f, "\nExpected <{}>, but end of file.", expected),
            ParseErr::IndErr { expected, actual, position } => match position {
                Some(pos) => write!(f, "\nExpected indentation of {}, but was {}, at ({}:{})\
                                    (next token: {})",
                                    expected, actual, pos.line, pos.pos, pos.token),
                None => write!(f, "\nExpected indentation of {}, but was {}.", expected, actual)
            }
        }
    }
}

impl error::Error for ParseErr {
    fn description(&self) -> &str {
        match self {
            ParseErr::ParseErr { .. } => "A parsing error occurred",
            ParseErr::UtilBodyErr => "Util module cannot have a body.",
            ParseErr::EOFErr { expected: _ } => "Expected token but end of file.",
            ParseErr::TokenErr { expected: _, actual: _ } => "Unexpected token encountered.",
            ParseErr::CustomErr { expected: _, actual: _ } => "Expected condition to be met.",
            ParseErr::CustomEOFErr { expected: _ } => "Expected condition to be met.",
            ParseErr::IndErr { expected: _, actual: _, position: _ } => "Unexpected indentation."
        }
    }

    fn source(&self) -> Option<&(error::Error + 'static)> { None }
}
