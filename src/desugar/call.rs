use crate::core::construct::Core;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;

pub fn desugar_call(node: &ASTNode, ctx: &Context, state: &State) -> Core {
    match node {
        ASTNode::Call { left, right } => match &right.node {
            ASTNode::Id { lit } => Core::FunctionCall {
                function: Box::from(desugar_node(&left, ctx, state)),
                args:     vec![Core::Id { lit: lit.clone() }]
            },
            ASTNode::Call { left: method, right } => match &method.node {
                ASTNode::Id { lit: method } => Core::MethodCall {
                    object: Box::from(desugar_node(&left, ctx, state)),
                    method: method.clone(),
                    args:   vec![desugar_node(&right, ctx, state)]
                },
                other => panic!("Chained method call must have identifier, was {:?}", other)
            },
            other => panic!("Expected call or id but was: {:?}", other)
        },
        other => panic!("Expected call flow but was: {:?}.", other)
    }
}
