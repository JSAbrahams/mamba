use crate::core::construct::Core;
use crate::desugarer::desugar::desugar_node;
use crate::parser::ast::ASTNodePos;

mod desugar;

pub fn desugar(input: &ASTNodePos) -> Core { desugar_node(&input) }
