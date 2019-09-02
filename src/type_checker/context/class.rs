use crate::parser::ast::ASTNodePos;
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::{TypeResult};

pub fn get_classes(ast_tree: &ASTNodePos) -> TypeResult<Vec<Type>> {
    Ok(vec![])
}
