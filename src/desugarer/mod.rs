use crate::core::construct::Core;
use crate::desugarer::expression::desugar_node;
use crate::parser::ast::ASTNodePos;

mod expression;

pub fn desugar(input: &ASTNodePos) -> Core { desugar_node(&input) }
