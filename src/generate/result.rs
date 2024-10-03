use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::common::position::Position;
use crate::common::result::{format_err, WithSource};
use crate::generate::ast::node::Core;
use crate::ASTTy;

pub type GenResult<T = Core> = Result<T, Box<UnimplementedErr>>;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct UnimplementedErr {
    pub position: Position,
    pub msg: String,
    pub source: Option<String>,
    pub path: Option<PathBuf>,
}

impl UnimplementedErr {
    pub fn new(ast: &ASTTy, msg: &str) -> UnimplementedErr {
        let msg = format!("The {msg} construct has not yet been implemented as of v{VERSION}");
        UnimplementedErr { position: ast.pos, msg, source: None, path: None }
    }
}

impl WithSource for UnimplementedErr {
    fn with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> UnimplementedErr {
        UnimplementedErr { source: source.clone(), path: path.clone(), ..self }
    }
}

impl Display for UnimplementedErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        format_err(f, &self.msg, &self.path, Some(self.position), &self.source, &[])
    }
}
