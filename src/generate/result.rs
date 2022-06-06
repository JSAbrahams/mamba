use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::{MAIN_SEPARATOR, PathBuf};

use crate::ASTTy;
use crate::common::position::Position;
use crate::common::result::WithSource;
use crate::generate::ast::node::Core;

pub type GenResult<T = Core> = Result<T, UnimplementedErr>;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub struct UnimplementedErr {
    pub position: Position,
    pub msg: String,
    pub source_line: Option<String>,
    pub path: Option<PathBuf>,
}

impl UnimplementedErr {
    pub fn new(ast: &ASTTy, msg: &str) -> UnimplementedErr {
        UnimplementedErr {
            position: ast.pos,
            msg: format!(
                "The {} construct has not yet been implemented as of v{}.",
                msg, VERSION
            ),
            source_line: None,
            path: None,
        }
    }
}

impl WithSource for UnimplementedErr {
    fn with_source(
        self,
        source: &Option<String>,
        path: &Option<PathBuf>,
    ) -> UnimplementedErr {
        UnimplementedErr {
            position: self.position,
            msg: self.msg.clone(),
            source_line: source.clone().map(|source| {
                source
                    .lines()
                    .nth(self.position.start.line as usize - 1)
                    .map_or(String::from("unknown"), String::from)
            }),
            path: path.clone(),
        }
    }
}

impl Display for UnimplementedErr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let path =
            self.path.clone().map_or(String::from("<unknown>"), |path| path.display().to_string());
        let path = path.strip_suffix(MAIN_SEPARATOR).unwrap_or(&path);

        write!(
            f,
            "--> {}:{}:{}\n     | {}\n{:3}  |- {}\n     | {}{}",
            path,
            self.position.start.line,
            self.position.start.pos,
            self.msg,
            self.position.start.line,
            self.source_line.clone().unwrap_or_else(|| String::from("<unknown>")),
            String::from_utf8(vec![b' '; self.position.start.pos as usize]).unwrap(),
            String::from_utf8(vec![b'^'; self.position.get_width() as usize]).unwrap()
        )
    }
}
