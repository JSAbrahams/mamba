use crate::core::construct::Core;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;

// TODO use context to check whether identifier is function or property
// Currently:
// a b => a.b , where a may be expression, b must be id
// a b c => a.b(c), where and c may be expression, b must be id
// a b c d => a.b(c.d) etc.
pub fn desugar_call(node: &ASTNode) -> Core {
    match node {
        ASTNode::Call { left, right } => match &right.node {
            ASTNode::Call { left: method, right: args } => match &method.node {
                ASTNode::Id { lit: method } => Core::MethodCall {
                    object: Box::from(desugar_node(&left)),
                    method: method.clone(),
                    args:   vec![desugar_node(&args)]
                },
                other => panic!("Chained method call must have identifier, was {:?}", other)
            },
            ASTNode::Id { lit } => Core::PropertyCall {
                object:   Box::from(desugar_node(&left)),
                property: lit.clone()
            },
            _ => match &left.node {
                ASTNode::Id { lit: method } => Core::MethodCall {
                    object: Box::from(Core::Empty),
                    method: method.clone(),
                    args:   vec![desugar_node(&right)]
                },
                other => panic!("desugar calls not that advanced yet: {:?}.", other)
            }
        },
        other => panic!("Expected call flow but was: {:?}.", other)
    }
}
