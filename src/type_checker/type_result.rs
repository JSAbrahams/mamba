use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::parser::ast::ASTNodePos;
use crate::type_checker::type_node::Type;

pub type TypeResult<T = Type> = std::result::Result<T, Vec<TypeErr>>;
pub type TypeResults =
    std::result::Result<Vec<(ASTNodePos, Option<String>, Option<PathBuf>)>, Vec<TypeErr>>;

#[derive(Debug)]
pub struct TypeErr {
    pub line:        i32,
    pub pos:         i32,
    pub width:       i32,
    pub msg:         String,
    pub path:        Option<PathBuf>,
    pub source_line: Option<String>
}

impl TypeErr {
    pub fn into_with_source(self, source: Option<String>, path: &Option<PathBuf>) -> TypeErr {
        TypeErr {
            line:        self.line,
            pos:         self.pos,
            width:       self.pos,
            msg:         self.msg.clone(),
            source_line: source.map(|source| {
                source
                    .lines()
                    .nth(self.line as usize - 1)
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
