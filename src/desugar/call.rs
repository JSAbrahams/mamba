use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::node::desugar_node;
use crate::desugar::result::DesugarResult;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::parse::ast::AST;
use crate::parse::ast::Node;

pub fn desugar_call(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    Ok(match &ast.node {
        Node::PropertyCall { instance, property } => Core::PropertyCall {
            object: Box::from(desugar_node(instance, imp, state)?),
            property: Box::from(desugar_node(property, imp, state)?),
        },
        Node::FunctionCall { name, args } => Core::FunctionCall {
            function: Box::from(desugar_node(name, imp, state)?),
            args: desugar_vec(args, imp, state)?,
        },
        other => panic!("Expected call flow but was: {:?}.", other)
    })
}
