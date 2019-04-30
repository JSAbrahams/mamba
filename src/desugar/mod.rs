use crate::core::construct::Core;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNodePos;

mod call;
mod common;
mod context;
mod control_flow;
mod definition;
mod node;

pub fn desugar(input: &ASTNodePos) -> Core { desugar_node(&input, &Context::new(), &State::new()) }
