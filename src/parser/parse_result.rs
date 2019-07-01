use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNodePos;

pub type ParseResult<T = Box<ASTNodePos>> = std::result::Result<T, ParseErr>;

#[derive(Debug)]
pub struct ParseErr {
    pub line:   i32,
    pub pos:    i32,
    pub width:  i32,
    pub msg:    String,
    pub causes: Vec<String>
}

impl ParseErr {
    /// Add cause of error to message as long as the maximum depth has not been
    /// exceeded.
    ///
    /// Else, it is ignored.
    pub fn clone_with_cause(&self, cause: &str) -> ParseErr {
        ParseErr {
            line:   self.line,
            pos:    self.pos,
            width:  self.width,
            msg:    self.msg.clone(),
            causes: {
                let mut new = self.causes.clone();
                new.push(format!("in {} \"{}\"", an_or_a(cause), cause));
                new
            }
        }
    }
}

fn comma_separated(tokens: &[Token]) -> String {
    let list = tokens.iter().fold(String::new(), |acc, token| acc + &format!("'{}', ", token));
    String::from(&list[0..if list.len() >= 2 { list.len() - 2 } else { 0 }])
}

fn an_or_a(parsing: &str) -> &str {
    match parsing.chars().next() {
        Some(c) if ['a', 'e', 'i', 'o', 'u'].contains(&c.to_ascii_lowercase()) => "an",
        _ => "a"
    }
}

pub fn expected_one_of(tokens: &[Token], actual: &TokenPos, parsing: &str) -> ParseErr {
    ParseErr {
        line:   actual.st_line,
        pos:    actual.st_pos,
        width:  actual.token.clone().width(),
        msg:    format!(
            "Expected one of [{}] while parsing {} \"{}\", but found token '{}'",
            comma_separated(tokens),
            an_or_a(parsing),
            parsing,
            actual.token
        ),
        causes: vec![]
    }
}

pub fn expected(expected: &Token, actual: &TokenPos, parsing: &str) -> ParseErr {
    ParseErr {
        line:   actual.st_line,
        pos:    actual.st_pos,
        width:  actual.token.clone().width(),
        msg:    format!(
            "Expected token '{}' while parsing {} \"{}\", but found token '{}'",
            expected,
            an_or_a(parsing),
            parsing,
            actual.token
        ),
        causes: vec![]
    }
}

pub fn custom(msg: &str, line: i32, pos: i32) -> ParseErr {
    ParseErr { line, pos, width: -1, msg: title_case(msg), causes: vec![] }
}

pub fn eof_expected_one_of(tokens: &[Token], parsing: &str) -> ParseErr {
    ParseErr {
        line:   -1,
        pos:    -1,
        width:  -1,
        msg:    format!(
            "Expected one of {} while parsing {} \"{}\", but end of file",
            comma_separated(tokens),
            an_or_a(parsing),
            parsing
        ),
        causes: vec![]
    }
}

fn title_case(s: &str) -> String {
    let mut tile_case = String::from(s);
    if let Some(first) = tile_case.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    tile_case.to_string()
}
