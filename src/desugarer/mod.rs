use crate::core::core_node::Core;
use crate::desugarer::expression::desugar_expression;
use crate::parser::ast_node::ASTNodePos;

mod expression;

pub fn desugar(input: ASTNodePos) -> Core { desugar_expression(&input) }
