use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::common::position::CaretPos;
use crate::parse::lex::token::{Lex, Token};

pub type LexResult<T = Vec<Lex>> = std::result::Result<T, LexErr>;

#[derive(Debug, Clone)]
pub struct LexErr {
    pub pos: CaretPos,
    pub token: Option<Token>,
    pub msg: String,
    pub source: Option<String>,
    pub path: Option<PathBuf>,
}

impl LexErr {
    pub fn new(pos: &CaretPos, token: Option<Token>, msg: &str) -> LexErr {
        LexErr { pos: pos.clone(), token, msg: String::from(msg), source: None, path: None }
    }

    pub fn into_with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> LexErr {
        LexErr { source: source.clone(), path: path.clone(), ..self.clone() }
    }
}

impl Display for LexErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let source_line = match &self.source {
            Some(source) => source
                .lines()
                .nth(self.pos.line as usize - 1)
                .unwrap_or("<unknown>"),
            None => "<unknown>"
        };

        write!(
            f,
            "--> {}:{}:{}\n     | {}\n{:3}  |- {}\n     | {}{}",
            self.path.clone().map_or(String::from("<unknown>"), |p| p.display().to_string()),
            self.pos.line,
            self.pos.pos,
            self.msg,
            self.pos.line,
            source_line,
            String::from_utf8(vec![b' '; self.pos.pos as usize]).unwrap(),
            String::from_utf8(vec![b'^'; self.token.clone().map_or(1, Token::width) as usize])
                .unwrap()
        )
    }
}
