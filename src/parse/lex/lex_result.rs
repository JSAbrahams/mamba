use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::common::position::CaretPos;
use crate::parse::lex::token::{Lex, Token};

pub type LexResult<T = Vec<Lex>> = std::result::Result<T, LexErr>;
pub type LexResults =
std::result::Result<Vec<(Vec<Lex>, Option<String>, Option<PathBuf>)>, Vec<LexErr>>;

#[derive(Debug, Clone)]
pub struct LexErr {
    pub pos: CaretPos,
    pub token: Option<Token>,
    pub msg: String,
    pub source_line: Option<String>,
    pub path: Option<PathBuf>,
}

impl LexErr {
    pub fn new(pos: &CaretPos, token: Option<Token>, msg: &str) -> LexErr {
        LexErr { pos: pos.clone(), token, msg: String::from(msg), source_line: None, path: None }
    }

    #[must_use]
    pub fn into_with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> LexErr {
        LexErr {
            pos: self.pos.clone(),
            token: self.token.clone(),
            msg: self.msg.clone(),
            source_line: source.clone().map(|source| {
                source
                    .lines()
                    .nth(self.pos.line as usize - 1)
                    .map_or(String::from("unknown"), String::from)
            }),
            path: path.clone(),
        }
    }
}

impl Display for LexErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "--> {}:{}:{}\n     | {}\n{:3}  |- {}\n     | {}{}",
            self.path.clone().map_or(String::from("<unknown>"), |p| p.display().to_string()),
            self.pos.line,
            self.pos.pos,
            self.msg,
            self.pos.line,
            self.source_line.clone().unwrap_or_else(|| String::from("<unknown>")),
            String::from_utf8(vec![b' '; self.pos.pos as usize]).unwrap(),
            String::from_utf8(vec![b'^'; self.token.clone().map_or(1, Token::width) as usize])
                .unwrap()
        )
    }
}
