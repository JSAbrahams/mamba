use crate::core::construct::Core;
use crate::parser::ast::ASTNodePos;

pub type DesugarResult<T = Core> = std::result::Result<T, UnimplementedErr>;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct UnimplementedErr {
    pub line: i32,
    pub pos:  i32,
    pub msg:  String
}

impl UnimplementedErr {
    pub fn new(node_pos: &ASTNodePos, msg: &str) -> UnimplementedErr {
        UnimplementedErr {
            line: node_pos.st_line,
            pos:  node_pos.st_pos,
            msg:  format!("The {} construct has not yet been implemented as of v{}.", msg, VERSION)
        }
    }
}
