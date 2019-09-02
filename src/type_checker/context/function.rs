use crate::parser::ast::ASTNodePos;
use crate::type_checker::context::field::Field;
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::TypeResult;

#[derive(Debug)]
pub struct Function {
    name:        String,
    arguments:   Vec<Field>,
    return_type: Option<Type>
}

pub fn get_functions(ast_tree: &ASTNodePos) -> TypeResult<Vec<Function>> { Ok(vec![]) }
