use crate::core::core::Core;
use crate::desugarer::expression::desugar_expression;
use crate::parser::ASTNodePos;
use std::collections::HashMap;
use std::collections::HashSet;

mod expression;

pub fn desugar(input: ASTNodePos) -> Core {
    desugar_expression(input)
}
