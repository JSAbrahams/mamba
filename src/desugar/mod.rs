use crate::core::construct::Core;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNodePos;

mod call;
mod control_flow;
mod definition;
mod node;
mod util;

pub fn desugar(input: &ASTNodePos) -> Core { desugar_node(&input) }
