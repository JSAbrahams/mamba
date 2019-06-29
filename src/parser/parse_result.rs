use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNodePos;

pub type ParseResult<T = Box<ASTNodePos>> = std::result::Result<T, ParseErr>;

#[derive(Debug)]
pub struct ParseErr {
    pub line:  i32,
    pub pos:   i32,
    pub width: i32,
    pub msg:   String
}

pub fn expected_construct(msg: &str, actual: &TokenPos) -> ParseErr {
    ParseErr {
        line:  actual.st_line,
        pos:   actual.st_pos,
        width: actual.token.clone().width(),
        msg:   format!("Expected a {}, but found token '{}'", msg, actual.token)
    }
}

pub fn expected(expected: &Token, actual: &TokenPos, msg: &str) -> ParseErr {
    ParseErr {
        line:  actual.st_line,
        pos:   actual.st_pos,
        width: actual.token.clone().width(),
        msg:   format!(
            "Expected token '{}' in {}, but found token '{}'",
            expected, msg, actual.token
        )
    }
}

pub fn custom(msg: &str, line: i32, pos: i32) -> ParseErr {
    ParseErr { line, pos, width: -1, msg: msg.to_string() }
}

pub fn eof(msg: &str) -> ParseErr {
    ParseErr { line: -1, pos: -1, width: -1, msg: msg.to_string() }
}

pub fn eof_expected(token: &Token) -> ParseErr {
    ParseErr {
        line:  -1,
        pos:   -1,
        width: -1,
        msg:   format!("Expected token '{}', but end of file", token)
    }
}
