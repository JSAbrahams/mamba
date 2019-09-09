use crate::common::position::EndPoint;
use crate::lexer::token::{Token, TokenPos};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub type LexResult<T = Vec<TokenPos>> = std::result::Result<T, LexErr>;
pub type LexResults =
    std::result::Result<Vec<(Vec<TokenPos>, Option<String>, Option<PathBuf>)>, Vec<LexErr>>;

#[derive(Debug, Clone)]
pub struct LexErr {
    pub start:       EndPoint,
    pub token:       Option<Token>,
    pub msg:         String,
    pub source_line: Option<String>,
    pub path:        Option<PathBuf>
}

impl LexErr {
    pub fn new(line: i32, pos: i32, token: Option<Token>, msg: &str) -> LexErr {
        LexErr {
            start: EndPoint { line, pos },
            token,
            msg: String::from(msg),
            source_line: None,
            path: None
        }
    }

    pub fn into_with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> LexErr {
        LexErr {
            start:       self.start.clone(),
            token:       self.token.clone(),
            msg:         self.msg.clone(),
            source_line: source.clone().map(|source| {
                source
                    .lines()
                    .nth(self.start.line as usize - 1)
                    .map_or(String::from("unknown"), String::from)
            }),
            path:        path.clone()
        }
    }
}

impl Display for LexErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "--> {:#?}:{}:{}
     | {}
{:3}  |- {}
     | {}{}",
            self.path.clone().map_or(String::from("<unknown>"), |path| format!("{:#?}", path)),
            self.start.line,
            self.start.pos,
            self.msg,
            self.start.line,
            self.source_line
                .clone()
                .map_or(String::from("<unknown>"), |line| format!("{:#?}", line)),
            String::from_utf8(vec![b' '; self.start.pos as usize]).unwrap(),
            String::from_utf8(vec![b'^'; self.token.clone().map_or(1, Token::width) as usize])
                .unwrap()
        )
    }
}
