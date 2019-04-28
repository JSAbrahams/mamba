use crate::core::construct::Core;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::desugar::util::desugar_vec;
use crate::parser::ast::ASTNode;

pub fn desugar_definition(node: &ASTNode, ctx: &Context, state: &State) -> Core {
    match node {
        ASTNode::Def { private, definition } => match &definition.node {
            // TODO do something with forward
            ASTNode::VariableDef { id_maybe_type, expression, .. } =>
                match (id_maybe_type, expression) {
                    (id, Some(expr)) => match desugar_node(&id, ctx, state) {
                        id @ Core::Tuple { .. } => Core::VarDef {
                            private: *private,
                            id:      Box::from(id),
                            right:   Box::from(desugar_node(&expr, ctx, state))
                        },
                        id => Core::VarDef {
                            private: *private,
                            id:      Box::from(id),
                            right:   Box::from(desugar_node(&expr, ctx, state))
                        }
                    },
                    (id, None) => Core::VarDef {
                        private: *private,
                        id:      Box::from(desugar_node(&id, ctx, state)),
                        right:   Box::from(Core::None)
                    }
                },
            ASTNode::FunDef { id, fun_args, body: expression, .. } => Core::FunDef {
                private: *private,
                id:      Box::from(desugar_node(&id, ctx, state)),
                args:    desugar_vec(&fun_args, ctx, state),
                body:    Box::from(match expression {
                    Some(expr) => desugar_node(&expr, ctx, state),
                    None => Core::Empty
                })
            },
            definition => panic!("invalid definition format: {:?}.", definition)
        },
        other => panic!("Expected control flow but was: {:?}.", other)
    }
}
