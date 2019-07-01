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

fn comma_separated(tokens: &[Token]) -> String {
    let list = tokens.iter().fold(String::new(), |acc, token| acc + &format!("'{}', ", token));
    String::from(&list[0..if list.len() >= 2 { list.len() - 2 } else { 0 }])
}

pub fn expected_one_of(tokens: &[Token], actual: &TokenPos, msg: &str) -> ParseErr {
    ParseErr {
        line:  actual.st_line,
        pos:   actual.st_pos,
        width: actual.token.clone().width(),
        msg:   format!(
            "Expected one of [{}] while parsing {}, but found token '{}'",
            comma_separated(tokens),
            msg,
            actual.token
        )
    }
}

pub fn expected(expected: &Token, actual: &TokenPos, msg: &str) -> ParseErr {
    ParseErr {
        line:  actual.st_line,
        pos:   actual.st_pos,
        width: actual.token.clone().width(),
        msg:   format!(
            "Expected token '{}' while parsing {}, but found token '{}'",
            expected, msg, actual.token
        )
    }
}

pub fn custom(msg: &str, line: i32, pos: i32) -> ParseErr {
    ParseErr { line, pos, width: -1, msg: title_case(msg) }
}

pub fn eof_expected_one_of(tokens: &[Token], msg: &str) -> ParseErr {
    ParseErr {
        line:  -1,
        pos:   -1,
        width: -1,
        msg:   format!(
            "Expected one of {} while parsing {}, but end of file",
            comma_separated(tokens),
            msg
        )
    }
}

fn title_case(s: &str) -> String {
    let mut tile_case = String::from(s);
    if let Some(first) = tile_case.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    tile_case.to_string()
}
