use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::check::ast::ASTTy;
use crate::common::position::Position;
use crate::common::result::{Cause, format_err, WithCause, WithSource};

pub type TypeResult<T = ASTTy> = Result<T, Vec<TypeErr>>;

pub trait TryFromPos<T>: Sized {
    fn try_from_pos(value: T, pos: Position) -> TypeResult<Self>;
}

#[derive(Debug, Clone, Eq)]
pub struct TypeErr {
    pub pos: Option<Position>,
    pub msg: String,
    pub path: Option<PathBuf>,
    pub source: Option<String>,
    causes: Vec<Cause>,
}

impl WithCause for TypeErr {
    fn with_cause(self, msg: &str, pos: Position) -> Self {
        TypeErr {
            causes: {
                let mut new_causes = self.causes.clone();
                new_causes.push(Cause::new(msg, pos));
                new_causes
            },
            ..self
        }
    }
}

impl Hash for TypeErr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
        self.msg.hash(state);
        self.path.hash(state);
    }
}

impl PartialEq for TypeErr {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.msg == other.msg && self.path == other.path
    }
}

impl From<TypeErr> for Vec<TypeErr> {
    fn from(type_err: TypeErr) -> Self {
        vec![type_err]
    }
}

impl TypeErr {
    /// New TypeErr with message at given position
    pub fn new(position: Position, msg: &str) -> TypeErr {
        TypeErr {
            pos: Some(position),
            msg: String::from(msg),
            path: None,
            source: None,
            causes: vec![],
        }
    }

    /// New TypeErr with message at random position
    pub fn new_no_pos(msg: &str) -> TypeErr {
        TypeErr {
            pos: None,
            msg: String::from(msg),
            path: None,
            source: None,
            causes: vec![],
        }
    }
}

impl WithSource for TypeErr {
    fn with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> TypeErr {
        TypeErr { source: source.clone(), path: path.clone(), ..self }
    }
}

impl Display for TypeErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        format_err(f, &self.msg, &self.path, self.pos, &self.source, &self.causes)
    }
}
