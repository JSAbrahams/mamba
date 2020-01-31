use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::common::position::Position;
use crate::type_checker::CheckInput;

pub type TypeResult<T> = std::result::Result<T, Vec<TypeErr>>;
pub type TypeResults = std::result::Result<Vec<CheckInput>, Vec<TypeErr>>;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TypeErr {
    pub position:      Option<Position>,
    pub msg:           String,
    pub path:          Option<PathBuf>,
    pub source_before: Option<String>,
    pub source_after:  Option<String>,
    pub source_line:   Option<String>
}

impl From<TypeErr> for Vec<TypeErr> {
    fn from(type_err: TypeErr) -> Self { vec![type_err] }
}

impl TypeErr {
    pub fn new(position: &Position, msg: &str) -> TypeErr {
        TypeErr {
            position:      Some(position.clone()),
            msg:           String::from(msg),
            path:          None,
            source_before: None,
            source_after:  None,
            source_line:   None
        }
    }

    pub fn new_no_pos(msg: &str) -> TypeErr {
        TypeErr {
            position:      None,
            msg:           String::from(msg),
            path:          None,
            source_before: None,
            source_after:  None,
            source_line:   None
        }
    }

    pub fn into_with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> TypeErr {
        let (source_before, source_line, source_after) = if let Some(position) = &self.position {
            if let Some(source) = source {
                (
                    if position.start.line >= 2 {
                        source.lines().nth(position.start.line as usize - 2)
                    } else {
                        None
                    },
                    if position.start.line >= 1 {
                        source.lines().nth(position.start.line as usize - 1)
                    } else {
                        None
                    },
                    source.lines().nth(position.start.line as usize)
                )
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        TypeErr {
            position:      self.position.clone(),
            msg:           self.msg,
            source_before: source_before.map(String::from),
            source_line:   source_line.map(String::from),
            source_after:  source_after.map(String::from),
            path:          path.clone()
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
                "--> {}:{}:{}\n     {}\n{}{:3}  |- {}\n     |  {}{}{}",
                path,
                position.start.line,
                position.start.pos,
                msg,
                self.source_before.clone().map_or_else(
                    || String::new(),
                    |src| if src.is_empty() {
                        String::new()
                    } else {
                        format!("{:3}  |  {}\n", position.start.line - 1, src)
                    },
                ),
                position.start.line,
                self.source_line.clone().unwrap_or_else(|| String::from("<unknown>")),
                String::from_utf8(vec![b' '; position.start.pos as usize - 1]).unwrap(),
                String::from_utf8(vec![b'^'; position.get_width() as usize]).unwrap(),
                self.source_after.clone().map_or(String::new(), |src| if src.is_empty() {
                    String::new()
                } else {
                    format!("\n{:3}  |  {}\n", position.start.line + 1, src)
                })
            )
        } else {
            write!(f, "--> {}\n     | {}", path, msg)
        }
    }
}
