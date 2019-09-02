use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::core::construct::Core;
use crate::parser::ast::ASTNodePos;

pub type DesugarResult<T = Core> = std::result::Result<T, UnimplementedErr>;
pub type DesugarResults =
    std::result::Result<Vec<(Core, Option<String>, Option<PathBuf>)>, Vec<UnimplementedErr>>;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct UnimplementedErr {
    pub line:        i32,
    pub pos:         i32,
    pub width:       i32,
    pub msg:         String,
    pub source_line: Option<String>,
    pub path:        Option<PathBuf>
}

impl UnimplementedErr {
    pub fn new(node_pos: &ASTNodePos, msg: &str) -> UnimplementedErr {
        UnimplementedErr {
            line:        node_pos.st_line,
            pos:         node_pos.st_pos,
            width:       node_pos.en_pos - node_pos.st_pos,
            msg:         format!(
                "The {} construct has not yet been implemented as of v{}.",
                msg, VERSION
            ),
            source_line: None,
            path:        None
        }
    }

    pub fn into_with_source(
        self,
        source: &Option<String>,
        path: &Option<PathBuf>
    ) -> UnimplementedErr {
        UnimplementedErr {
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

impl Display for UnimplementedErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "--> {}:{}:{}
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
