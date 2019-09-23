use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Imports;
use crate::desugar::context::State;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::parser::ast::Node;
use crate::parser::ast::AST;

pub fn desugar_call(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    Ok(match &ast.node {
        Node::PropertyCall { instance, property } => Core::PropertyCall {
            object:   Box::from(desugar_node(instance, imp, state)?),
            property: Box::from(desugar_node(property, imp, state)?)
        },
        Node::FunctionCall { name, args } => Core::FunctionCall {
            function: Box::from(desugar_node(name, imp, state)?),
            args:     desugar_vec(args, imp, state)?
        },
        other => panic!("Expected call flow but was: {:?}.", other)
    })
}
