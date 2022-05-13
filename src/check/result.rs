use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::check::CheckInput;
use crate::common::position::Position;

pub type TypeResult<T> = std::result::Result<T, Vec<TypeErr>>;
pub type TypeResults = std::result::Result<Vec<CheckInput>, Vec<TypeErr>>;

#[derive(Debug, Clone, Eq)]
pub struct TypeErr {
    pub position:      Option<Position>,
    pub msg:           String,
    pub path:          Option<PathBuf>,
    pub source_before: Option<String>,
    pub source_after:  Option<String>,
    pub source_line:   Option<String>
}

impl Hash for TypeErr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.msg.hash(state);
        self.path.hash(state);
    }
}

impl PartialEq for TypeErr {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.msg == other.msg && self.path == other.path
    }
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

    #[must_use]
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
            let mut string = String::from(self.msg.trim());
            if string.ends_with('\n') {
                string.remove(string.len() - 1);
            }
            string.replace('\n', "\n   > ")
        };

        if let Some(position) = self.position.clone() {
            write!(
                f,
                "{}\n --> {}:{}:{}\n{}{:4} | {}\n       {}{}{}",
                msg,
                path,
                position.start.line,
                position.start.pos,
                self.source_before.clone().map_or_else(String::new, |src| if src.is_empty() {
                    String::new()
                } else {
                    format!("{:4} | {}\n", position.start.line - 1, src)
                }),
                position.start.line,
                self.source_line.clone().unwrap_or_else(|| String::from("<unknown>")),
                String::from_utf8(vec![b' '; position.start.pos as usize - 1]).unwrap(),
                String::from_utf8(vec![b'^'; position.get_width() as usize]).unwrap(),
                self.source_after.clone().map_or(String::new(), |src| if src.is_empty() {
                    String::new()
                } else {
                    format!("\n{:4} | {}\n", position.start.line + 1, src)
                })
            )
        } else {
            let path = if let Some(path) = &self.path {
                format!("\n --> {}", path.display())
            } else {
                String::new()
            };

            write!(f, "{}{}", msg, path)
        }
    }
}
