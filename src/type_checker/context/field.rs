use crate::parser::ast::ASTNodePos;
use crate::type_checker::type_node::Type;

#[derive(Debug)]
pub struct Field {
    id:       Type,
    location: Vec<String>,
    mutable:  bool,
    public:   bool,
    ty:       Type
}

impl Field {
    pub fn new(node_pos: &ASTNodePos) -> Result<Field, String> {
        Ok(Field {
            id:       Type::Empty,
            location: vec![],
            mutable:  false,
            public:   false,
            ty:       Type::Empty
        })
    }
}
