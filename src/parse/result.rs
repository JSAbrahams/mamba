use std::cmp::{max, min};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::common::result::{an_or_a, format_err, Cause, WithCause, WithSource};
use crate::parse::ast::AST;
use crate::parse::lex::result::LexErr;
use crate::parse::lex::token::Lex;
use crate::parse::lex::token::Token;

const SYNTAX_ERR_MAX_DEPTH: usize = 1;

pub type ParseResult<T = Box<AST>> = Result<T, Box<ParseErr>>;

#[derive(Debug, Clone)]
pub struct ParseErr {
    pub pos: Position,
    pub msg: String,
    pub source: Option<String>,
    pub path: Option<PathBuf>,
    pub causes: Vec<Cause>,
}

impl WithCause for ParseErr {
    fn with_cause(self, msg: &str, pos: Position) -> Self {
        let causes = {
            let mut new_causes = self.causes.clone();
            let msg = format!("While parsing {}{msg}", an_or_a(msg));

            new_causes.push(Cause::new(&msg, pos));
            new_causes
        };
        ParseErr { causes, ..self }
    }
}

impl WithSource for ParseErr {
    fn with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> ParseErr {
        ParseErr {
            source: source.clone(),
            path: path.clone(),
            ..self
        }
    }
}

impl From<LexErr> for ParseErr {
    fn from(lex_err: LexErr) -> Self {
        ParseErr {
            pos: Position::from(lex_err.pos),
            msg: lex_err.msg,
            source: lex_err.source,
            path: lex_err.path,
            causes: vec![],
        }
    }
}

pub fn expected_one_of(tokens: &[Token], actual: &Lex, parsing: &str) -> ParseErr {
    let msg = format!(
        "Expected one of [{}] while parsing {}{parsing}, but found token '{}'",
        comma_delm(tokens),
        an_or_a(parsing),
        actual.token
    );
    ParseErr {
        pos: actual.pos,
        msg,
        source: None,
        path: None,
        causes: vec![],
    }
}

pub fn expected(expected: &Token, actual: &Lex, parsing: &str) -> ParseErr {
    let msg = format!(
        "Expected {}{expected} token while parsing {}{parsing}, but found {}",
        an_or_a(expected),
        an_or_a(parsing),
        actual.token
    );
    ParseErr {
        pos: actual.pos,
        msg,
        source: None,
        path: None,
        causes: vec![],
    }
}

pub fn custom(msg: &str, position: Position) -> ParseErr {
    ParseErr {
        pos: position,
        msg: title_case(msg),
        source: None,
        path: None,
        causes: vec![],
    }
}

pub fn eof_expected_one_of(tokens: &[Token], parsing: &str) -> ParseErr {
    ParseErr {
        pos: Position::invisible(),
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
            _ => format!(
                "Expected a token while parsing {}{parsing}",
                an_or_a(parsing)
            ),
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
        let causes = &self.causes[0..min(
            max(self.causes.len() as i32 - 1, 0) as usize,
            SYNTAX_ERR_MAX_DEPTH,
        )];
        format_err(
            f,
            &self.msg,
            &self.path,
            Some(self.pos),
            &self.source,
            causes,
        )
    }
}
