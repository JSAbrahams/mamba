use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;

pub fn desugar_definition(node: &ASTNode, ctx: &Context, state: &State) -> Core {
    match node {
        ASTNode::Def { private, definition } => match &definition.node {
            // TODO do something with forward
            ASTNode::VariableDef { id_maybe_type, expression, .. } =>
                match (id_maybe_type, expression) {
                    (id, Some(expr)) => match id.node.clone() {
                        ASTNode::Tuple { elements } => Core::VarDef {
                            private: *private,
                            id:      Box::from(desugar_node(id, ctx, state)),
                            right:   Box::from(desugar_node(&expr, ctx, &State {
                                tup:         elements.len(),
                                expect_expr: true
                            }))
                        },
                        _ => Core::VarDef {
                            private: *private,
                            id:      Box::from(desugar_node(id, ctx, state)),
                            right:   Box::from(desugar_node(&expr, ctx, &State {
                                tup:         1,
                                expect_expr: true
                            }))
                        }
                    },
                    (id, None) => match desugar_node(id, ctx, state) {
                        Core::Tuple { elements } => {
                            let length = elements.len();
                            Core::VarDef {
                                private: *private,
                                id:      Box::from(Core::Tuple { elements }),
                                right:   Box::from(Core::Tuple {
                                    elements: vec![Core::None; length]
                                })
                            }
                        }
                        other => Core::VarDef {
                            private: *private,
                            id:      Box::from(other),
                            right:   Box::from(Core::None)
                        }
                    }
                },
            ASTNode::FunDef { id, fun_args, body: expression, .. } => Core::FunDef {
                private: *private,
                id:      Box::from(desugar_node(&id, ctx, state)),
                args:    desugar_vec(&fun_args, ctx, state),
                body:    Box::from(match expression {
                    Some(expr) => desugar_node(&expr, ctx, &State {
                        tup:         state.tup,
                        expect_expr: true
                    }),
                    None => Core::Empty
                })
            },
            definition => panic!("invalid definition format: {:?}.", definition)
        },
        other => panic!("Expected control flow but was: {:?}.", other)
    }
}
