use crate::core::core::Core;
use crate::desugarer::expression::desugar_expression;
use crate::parser::ASTNodePos;

mod expression;

pub fn desugar(input: ASTNodePos) -> Core { desugar_expression(&input) }
