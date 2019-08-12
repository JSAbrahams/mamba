use crate::parser::ast::ASTNodePos;
use crate::type_checker::context::class::Interface;
use crate::type_checker::type_node::Type;

#[derive(Debug)]
pub struct Function {
    id:       Type,
    location: Vec<String>,
    public:   bool,
    args:     Vec<FunctionArg>,
    ret:      Type,
    raises:   Interface
}

#[derive(Debug)]
pub struct FunctionArg {
    id: String,
    ty: Type
}

impl Function {
    pub fn new(node_pos: &ASTNodePos) -> Result<Function, String> {
        Ok(Function {
            id:       Type::Empty,
            location: vec![],
            public:   false,
            args:     vec![],
            ret:      Type::Empty,
            raises:   Interface::new(node_pos)?
        })
    }
}
