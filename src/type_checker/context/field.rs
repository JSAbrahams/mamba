use crate::type_checker::type_node::Type;
use crate::parser::ast::ASTNodePos;
use crate::type_checker::type_result::TypeResult;

pub struct Field {
    name: String,
    ty:   Type
}

pub fn get_fields(ast_tree: &ASTNodePos) -> TypeResult<Vec<Field>> {
    Ok(vec![])
}
