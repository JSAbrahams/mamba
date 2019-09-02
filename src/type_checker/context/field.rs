use crate::parser::ast::ASTNodePos;
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::TypeResult;

#[derive(Debug)]
pub struct Field {
    name: String,
    ty:   Type
}

pub fn get_fields(ast_tree: &ASTNodePos) -> TypeResult<Vec<Field>> { Ok(vec![]) }
