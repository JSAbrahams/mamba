use crate::core::construct::Core;
use crate::desugar::node::desugar_node;
use crate::desugar::util::desugar_vec;
use crate::parser::ast::ASTNode;

pub fn desugar_definition(node: &ASTNode) -> Core {
    match node {
        ASTNode::Def { private, definition } => match &definition.node {
            // TODO do something with forward
            ASTNode::VariableDef { id_maybe_type, expression, .. } =>
                match (id_maybe_type, expression) {
                    (id, Some(expr)) => Core::VarDef {
                        private: *private,
                        id:      Box::from(desugar_node(&id)),
                        right:   Box::from(desugar_node(&expr))
                    },
                    (id, None) => desugar_node(&id)
                },
            ASTNode::FunDef { id, fun_args, body: expression, .. } => Core::FunDef {
                private: *private,
                id:      Box::from(desugar_node(&id)),
                args:    desugar_vec(&fun_args),
                body:    Box::from(match expression {
                    Some(expr) => desugar_node(&expr),
                    None => Core::Empty
                })
            },
            definition => panic!("invalid definition format: {:?}.", definition)
        },
        other => panic!("Expected control flow but was: {:?}.", other)
    }
}
