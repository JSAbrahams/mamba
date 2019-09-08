use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::common::position::Position;
use crate::core::construct::Core;
use crate::parser::ast::ASTNodePos;

pub type DesugarResult<T = Core> = std::result::Result<T, UnimplementedErr>;
pub type DesugarResults =
    std::result::Result<Vec<(Core, Option<String>, Option<PathBuf>)>, Vec<UnimplementedErr>>;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct UnimplementedErr {
    pub position:    Position,
    pub msg:         String,
    pub source_line: Option<String>,
    pub path:        Option<PathBuf>
}

impl UnimplementedErr {
    pub fn new(node_pos: &ASTNodePos, msg: &str) -> UnimplementedErr {
        UnimplementedErr {
            position:    node_pos.position.clone(),
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
            position:    self.position.clone(),
            msg:         self.msg.clone(),
            source_line: source.clone().map(|source| {
                source
                    .lines()
                    .nth(self.position.start.line as usize - 1)
                    .map_or(String::from("unknown"), String::from)
            }),
            path:        path.clone()
        }
    }
}

impl Display for UnimplementedErr {
    // TODO handle multi-line errors
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "--> {}:{}:{}
     | {}
{:3}  |- {}
     | {}{}",
            self.path.clone().map_or(String::from("<unknown>"), |path| format!("{:#?}", path)),
            self.position.start.line,
            self.position.start.pos,
            self.msg,
            self.position.start.line,
            self.source_line
                .clone()
                .map_or(String::from("<unknown>"), |line| format!("{:#?}", line)),
            String::from_utf8(vec![b' '; self.position.end.pos as usize]).unwrap(),
            String::from_utf8(vec![b'^'; self.position.get_width() as usize]).unwrap()
        )
    }
}
