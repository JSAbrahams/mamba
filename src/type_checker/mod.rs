use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;

pub fn type_check(input: ASTNodePos) -> ASTNode { input.node }
