use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::common::position::Position;
use crate::parser::ast::ASTNodePos;
use crate::type_checker::type_node::Type;

pub type TypeResult<T = Type> = std::result::Result<T, Vec<TypeErr>>;
pub type TypeResults =
    std::result::Result<Vec<(ASTNodePos, Option<String>, Option<PathBuf>)>, Vec<TypeErr>>;

#[derive(Debug)]
pub struct TypeErr {
    pub position:    Position,
    pub msg:         String,
    pub path:        Option<PathBuf>,
    pub source_line: Option<String>
}

impl TypeErr {
    pub fn new(position: Position, msg: &str) -> TypeErr {
        TypeErr { position, msg: String::from(msg), path: None, source_line: None }
    }

    pub fn into_with_source(self, source: Option<String>, path: &Option<PathBuf>) -> TypeErr {
        TypeErr {
            position:    self.position.clone(),
            msg:         self.msg.clone(),
            source_line: source.map(|source| {
                source
                    .lines()
                    .nth(self.position.line as usize - 1)
                    .map_or(String::from("unknown"), String::from)
            }),
            path:        path.clone()
        }
    }
}

impl Display for TypeErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "--> {:#?}:{}:{}
     | {}
{:3}  |- {}
     | {}{}",
            self.path.clone().map_or(String::from("<unknown>"), |path| format!("{:#?}", path)),
            self.position.line,
            self.position.pos,
            self.msg,
            self.position.line,
            self.source_line
                .clone()
                .map_or(String::from("<unknown>"), |line| format!("{:#?}", line)),
            String::from_utf8(vec![b' '; self.position.pos as usize]).unwrap(),
            String::from_utf8(vec![b'^'; self.position.width as usize]).unwrap()
        )
    }
}
