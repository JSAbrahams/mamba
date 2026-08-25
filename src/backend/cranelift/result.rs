use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::check::ast::ASTTy;
use crate::common::position::Position;
use crate::common::result::{format_err, WithSource};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub type BackendResult<T> = Result<T, Box<BackendErr>>;

#[derive(Debug, Clone)]
pub struct BackendErr {
    pub position: Position,
    pub msg: String,
    pub source: Option<String>,
    pub path: Option<PathBuf>,
}

impl BackendErr {
    /// A construct that the Cranelift backend doesn't (yet) support -- unlike the Python backend,
    /// which aims to eventually support the whole language, this one is intentionally scoped to a
    /// small subset (see `src/backend/cranelift/lower.rs`), so this is expected long-term, not
    /// just a temporary gap.
    pub fn unimplemented(ast: &ASTTy, msg: &str) -> Box<BackendErr> {
        let msg = format!(
            "The {msg} construct is not supported by the machine-code backend (v{VERSION})"
        );
        Box::from(BackendErr {
            position: ast.pos,
            msg,
            source: None,
            path: None,
        })
    }

    pub fn new(position: Position, msg: &str) -> Box<BackendErr> {
        Box::from(BackendErr {
            position,
            msg: String::from(msg),
            source: None,
            path: None,
        })
    }
}

impl WithSource for BackendErr {
    fn with_source(self, source: &Option<String>, path: &Option<PathBuf>) -> BackendErr {
        BackendErr {
            source: source.clone(),
            path: path.clone(),
            ..self
        }
    }
}

impl Display for BackendErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        format_err(
            f,
            &self.msg,
            &self.path,
            Some(self.position),
            &self.source,
            &[],
        )
    }
}
