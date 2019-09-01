use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::lexer::token::TokenPos;

pub type LexResult<T = Vec<TokenPos>> = std::result::Result<T, LexErr>;

#[derive(Debug)]
pub struct LexErr {
    pub line:        i32,
    pub pos:         i32,
    pub width:       i32,
    pub msg:         String,
    pub source_line: Option<String>,
    pub path:        Option<PathBuf>
}

impl LexErr {
    pub fn new(line: i32, pos: i32, width: i32, msg: &str) -> LexErr {
        LexErr { line, pos, width, msg: String::from(msg), source_line: None, path: None }
    }

    pub fn into_with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> LexErr {
        LexErr {
            line:        self.line,
            pos:         self.pos,
            width:       self.pos,
            msg:         self.msg.clone(),
            source_line: source.clone().map(|source| {
                source
                    .lines()
                    .nth(self.line as usize - 1)
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
            self.line,
            self.pos,
            self.msg,
            self.line,
            self.source_line
                .clone()
                .map_or(String::from("<unknown>"), |line| format!("{:#?}", line)),
            String::from_utf8(vec![b' '; self.pos as usize]).unwrap(),
            String::from_utf8(vec![b'^'; self.width as usize]).unwrap()
        )
    }
}
