use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::common::position::Position;
use crate::type_checker::CheckInput;

pub type TypeResult<T> = std::result::Result<T, Vec<TypeErr>>;
pub type TypeResults = std::result::Result<Vec<CheckInput>, Vec<TypeErr>>;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TypeErr {
    pub position:    Option<Position>,
    pub msg:         String,
    pub path:        Option<PathBuf>,
    pub source_line: Option<String>
}

impl From<TypeErr> for Vec<TypeErr> {
    fn from(type_err: TypeErr) -> Self { vec![type_err] }
}

impl TypeErr {
    pub fn new(position: &Position, msg: &str) -> TypeErr {
        TypeErr {
            position:    Some(position.clone()),
            msg:         String::from(msg),
            path:        None,
            source_line: None
        }
    }

    pub fn new_no_pos(msg: &str) -> TypeErr {
        TypeErr {
            position:    None,
            msg:         String::from(msg),
            path:        None,
            source_line: None
        }
    }

    pub fn into_with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> TypeErr {
        TypeErr {
            position:    self.position.clone(),
            msg:         self.msg.clone(),
            source_line: if let Some(position) = self.position {
                source.clone().map(|source| {
                    source
                        .lines()
                        .nth(position.start.line as usize - 1)
                        .map_or(String::from("unknown"), String::from)
                })
            } else {
                Some(String::from("unknown"))
            },
            path:        path.clone()
        }
    }
}

impl Display for TypeErr {
    // TODO deal with Positions that cover multiple lines
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let path = self.path.clone().map_or(String::from("<unknown>"), |p| p.display().to_string());
        let msg = {
            let mut string = self.msg.replace("\n", "\n     | |  ");
            if string.ends_with('|') {
                string.remove(string.len() - 2);
            }
            string
        };

        if let Some(position) = self.position.clone() {
            write!(
                f,
                "--> {}:{}:{}\n     | {}\n{:3}  |- {}\n     |  {}{}",
                path,
                position.start.line,
                position.start.pos,
                msg,
                position.start.line,
                self.source_line.clone().unwrap_or_else(|| String::from("<unknown>")),
                String::from_utf8(vec![b' '; position.start.pos as usize - 1]).unwrap(),
                String::from_utf8(vec![b'^'; position.get_width() as usize]).unwrap()
            )
        } else {
            write!(f, "--> {}\n     | {}", path, msg)
        }
    }
}
