use std::cmp::min;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNodePos;

const SYNTAX_ERR_MAX_DEPTH: usize = 2;

pub type ParseResult<T = Box<ASTNodePos>> = std::result::Result<T, ParseErr>;

#[derive(Debug)]
pub struct ParseErr {
    pub line:   i32,
    pub pos:    i32,
    pub width:  i32,
    pub msg:    String,
    pub source: Option<String>,
    pub path:   Option<PathBuf>,
    pub causes: Vec<Cause>
}

#[derive(Debug, Clone)]
pub struct Cause {
    pub line:  i32,
    pub pos:   i32,
    pub cause: String
}

impl Cause {
    pub fn new(cause: &str, line: i32, pos: i32) -> Cause {
        Cause { line, pos, cause: String::from(cause) }
    }
}

impl ParseErr {
    pub fn clone_with_cause(&self, cause: &str, line: i32, pos: i32) -> ParseErr {
        ParseErr {
            line:   self.line,
            pos:    self.pos,
            width:  self.width,
            msg:    self.msg.clone(),
            causes: {
                let mut new_causes = self.causes.clone();
                new_causes.push(Cause::new(cause, line, pos));
                new_causes
            },
            source: self.source.clone(),
            path:   self.path.clone()
        }
    }

    pub fn into_with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> ParseErr {
        ParseErr {
            line:   self.line,
            pos:    self.pos,
            width:  self.pos,
            msg:    self.msg,
            causes: self.causes,
            source: source.clone(),
            path:   path.clone()
        }
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
        source: None,
        path:   None,
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
        source: None,
        path:   None,
        causes: vec![]
    }
}

pub fn custom(msg: &str, line: i32, pos: i32) -> ParseErr {
    ParseErr {
        line,
        pos,
        width: -1,
        msg: title_case(msg),
        source: None,
        path: None,
        causes: vec![]
    }
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
        source: None,
        path:   None,
        causes: vec![]
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

fn title_case(s: &str) -> String {
    let mut tile_case = String::from(s);
    if let Some(first) = tile_case.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    tile_case.to_string()
}

impl Display for ParseErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let cause_formatter = &self.causes[0..min(self.causes.len(), SYNTAX_ERR_MAX_DEPTH)]
            .iter()
            .rev()
            .fold(String::new(), |acc, cause| {
                let source_line = match &self.source {
                    Some(source) =>
                        source.lines().nth(cause.line as usize - 1).unwrap_or("<unknown>"),
                    None => "<unknown>"
                };

                acc + &format!(
                    "{:3}  |- {}\n     | {}^ in {} ({}:{})\n",
                    cause.line,
                    source_line,
                    String::from_utf8(vec![b' '; cause.pos as usize]).unwrap(),
                    cause.cause,
                    cause.line,
                    cause.pos,
                )
            });

        let source_line = match &self.source {
            Some(source) => source.lines().nth(self.line as usize - 1).unwrap_or("<unknown>"),
            None => "<unknown>"
        };

        write!(
            f,
            "--> {:#?}:{}:{}
     | {}
{}
{:3}  |- {}
     | {}{}",
            self.path.clone().map_or(String::from("<unknown>"), |path| format!("{:#?}", path)),
            self.line,
            self.pos,
            self.msg,
            cause_formatter,
            self.line,
            source_line,
            String::from_utf8(vec![b' '; self.pos as usize]).unwrap(),
            String::from_utf8(vec![b'^'; self.width as usize]).unwrap()
        )
    }
}
