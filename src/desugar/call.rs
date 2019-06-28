use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;

pub fn desugar_call(node: &ASTNode, ctx: &mut Context, state: &State) -> Core {
    match node {
        ASTNode::PropertyCall { instance, property } => Core::PropertyCall {
            object:   Box::from(desugar_node(instance, ctx, state)),
            property: Box::from(desugar_node(property, ctx, state))
        },
        ASTNode::FunctionCall { name, args } => Core::FunctionCall {
            function: Box::from(desugar_node(name, ctx, state)),
            args:     desugar_vec(args, ctx, state)
        },
        other => panic!("Expected call flow but was: {:?}.", other)
    }
}
