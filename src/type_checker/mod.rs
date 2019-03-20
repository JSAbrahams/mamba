use crate::parser::ast_node::ASTNode;
use crate::parser::ast_node::ASTNodePos;

pub fn type_check(input: ASTNodePos) -> ASTNode { input.node }
