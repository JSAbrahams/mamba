use crate::core::construct::Core;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;

pub fn desugar_call(node: &ASTNode, ctx: &Context, state: &State) -> Core {
    match node {
        ASTNode::Call { left, right } => match &right.node {
            ASTNode::Id { lit } => Core::PropertyCall {
                object:   Box::from(desugar_node(&left, ctx, state)),
                property: lit.clone()
            },
            ASTNode::Call { left: method, right } => match &method.node {
                ASTNode::Id { lit: method } => Core::MethodCall {
                    object: Box::from(desugar_node(&left, ctx, state)),
                    method: method.clone(),
                    args:   vec![desugar_node(&right, ctx, state)]
                },
                other => panic!("Chained method call must have identifier, was {:?}", other)
            },
            _ => match &left.node {
                ASTNode::Id { lit: method } => Core::MethodCall {
                    object: Box::from(Core::Empty),
                    method: method.clone(),
                    args:   vec![desugar_node(&right, ctx, state)]
                },
                other => panic!("desugar calls not that advanced yet: {:?}.", other)
            }
        },
        other => panic!("Expected call flow but was: {:?}.", other)
    }
}
