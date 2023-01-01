use std::cmp::min;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::{MAIN_SEPARATOR, PathBuf};

use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::common::result::{an_or_a, WithSource};
use crate::parse::ast::AST;
use crate::parse::lex::result::LexErr;
use crate::parse::lex::token::Lex;
use crate::parse::lex::token::Token;

const SYNTAX_ERR_MAX_DEPTH: usize = 1;

pub type ParseResult<T = Box<AST>> = Result<T, Box<ParseErr>>;

#[derive(Debug, Clone)]
pub struct ParseErr {
    pub position: Position,
    pub msg: String,
    pub source: Option<String>,
    pub path: Option<PathBuf>,
    pub causes: Vec<Cause>,
}

#[derive(Debug, Clone)]
pub struct Cause {
    pub position: Position,
    pub cause: String,
}

impl Cause {
    pub fn new(cause: &str, position: Position) -> Cause {
        Cause { position, cause: String::from(cause) }
    }
}

impl ParseErr {
    #[must_use]
    pub fn clone_with_cause(&self, cause: &str, position: Position) -> ParseErr {
        ParseErr {
            position: self.position,
            msg: self.msg.clone(),
            causes: {
                let mut new_causes = self.causes.clone();
                new_causes.push(Cause::new(cause, position));
                new_causes
            },
            source: self.source.clone(),
            path: self.path.clone(),
        }
    }
}

impl WithSource for ParseErr {
    fn with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> ParseErr {
        ParseErr {
            position: self.position,
            msg: self.msg,
            causes: self.causes,
            source: source.clone(),
            path: path.clone(),
        }
    }
}

impl From<LexErr> for ParseErr {
    fn from(lex_err: LexErr) -> Self {
        ParseErr {
            position: Position::from(lex_err.pos),
            msg: lex_err.msg,
            source: lex_err.source,
            path: lex_err.path,
            causes: vec![],
        }
    }
}

pub fn expected_one_of(tokens: &[Token], actual: &Lex, parsing: &str) -> ParseErr {
    ParseErr {
        position: actual.pos,
        msg: format!(
            "Expected one of [{}] while parsing {}{parsing}, but found token '{}'",
            comma_delm(tokens),
            an_or_a(parsing),
            actual.token
        ),
        source: None,
        path: None,
        causes: vec![],
    }
}

pub fn expected(expected: &Token, actual: &Lex, parsing: &str) -> ParseErr {
    ParseErr {
        position: actual.pos,
        msg: format!(
            "Expected {}{expected} token while parsing {}{parsing}, but found {}",
            an_or_a(expected),
            an_or_a(parsing),
            actual.token
        ),
        source: None,
        path: None,
        causes: vec![],
    }
}

pub fn custom(msg: &str, position: Position) -> ParseErr {
    ParseErr { position, msg: title_case(msg), source: None, path: None, causes: vec![] }
}

pub fn eof_expected_one_of(tokens: &[Token], parsing: &str) -> ParseErr {
    ParseErr {
        position: Position::default(),
        msg: match tokens {
            tokens if tokens.len() > 1 => format!(
                "Expected one of [{}] tokens while parsing {}{parsing}",
                comma_delm(tokens),
                an_or_a(parsing),
            ),
            tokens if tokens.len() == 1 => format!(
                "Expected a {} token while parsing {}{parsing}",
                comma_delm(tokens),
                an_or_a(parsing),
            ),
            _ => format!("Expected a token while parsing {}{parsing}", an_or_a(parsing)),
        },
        source: None,
        path: None,
        causes: vec![],
    }
}

fn title_case(s: &str) -> String {
    let mut tile_case = String::from(s);
    if let Some(first) = tile_case.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    tile_case
}

impl Display for ParseErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // The first cause is the error itself
        let cause_formatter = &self.causes[0..min(self.causes.len(), SYNTAX_ERR_MAX_DEPTH)]
            .iter()
            .rev()
            .skip(1)
            .fold(String::new(), |acc, cause| {
                let source_line = match &self.source {
                    Some(source) => source
                        .lines()
                        .nth(cause.position.start.line - 1)
                        .unwrap_or("<unknown>"),
                    None => "<unknown>",
                };

                acc + &format!(
                    "{:3}  |- {source_line}\n     | {}^ in {} ({}:{})\n",
                    cause.position.start.line,
                    String::from_utf8(vec![b' '; cause.position.start.pos]).unwrap(),
                    cause.cause,
                    cause.position.start.line,
                    cause.position.start.pos,
                )
            });

        let path = self.path.as_ref().map_or("<unknown>", |path| path.to_str().unwrap_or_default());
        let source_line = match &self.source {
            Some(source) => {
                source.lines().nth(self.position.start.line - 1).unwrap_or("<unknown>")
            }
            None => "<unknown>",
        };

        write!(
            f,
            "{}\n --> {}:{}:{}\n {:3} |- {}\n     | {}{}\n{}",
            self.msg,
            path.strip_suffix(MAIN_SEPARATOR).unwrap_or(path),
            self.position.start.line,
            self.position.start.pos,
            self.position.start.line,
            source_line,
            String::from_utf8(vec![b' '; self.position.start.pos]).unwrap(),
            String::from_utf8(vec![b'^'; self.position.get_width()]).unwrap(),
            cause_formatter,
        )
    }
}
