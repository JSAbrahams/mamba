use crate::parser::ast::ASTNodePos;

#[derive(Clone, Debug)]
pub struct Position {
    pub line:  i32,
    pub pos:   i32,
    pub width: i32
}

impl From<&ASTNodePos> for Position {
    fn from(node_pos: &ASTNodePos) -> Self {
        Position {
            line:  node_pos.st_line,
            pos:   node_pos.st_pos,
            width: node_pos.en_pos - node_pos.st_pos
        }
    }
}
