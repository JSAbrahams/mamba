use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;

pub fn desugar_definition(node: &ASTNode, ctx: &Context, state: &State) -> Core {
    match node {
        ASTNode::Def { private, definition } => match &definition.node {
            ASTNode::VariableDef { id_maybe_type, expression, forward, .. } => {
                let id = desugar_node(id_maybe_type, ctx, state);
                let new_state = &State {
                    tup:         match id.clone() {
                        Core::Tuple { elements } => elements.len(),
                        _ => 1
                    },
                    expect_expr: true
                };

                let item = Core::VarDef {
                    private: *private,
                    id:      Box::from(id.clone()),
                    right:   match (id.clone(), expression) {
                        (_, Some(expr)) => Box::from(desugar_node(&expr, ctx, &new_state)),
                        (Core::Tuple { elements }, None) =>
                            Box::from(Core::Tuple { elements: vec![Core::None; elements.len()] }),
                        (_, None) => Box::from(Core::None)
                    }
                };

                let mut statements = vec![item];
                forward.iter().for_each(|node_pos| match (&id, &node_pos.node) {
                    (Core::Id { lit: item_lit }, ASTNode::Id { lit: method_lit }) =>
                        statements.push(forward_def(item_lit.clone(), method_lit.clone())),
                    (Core::Id { .. }, other) =>
                        panic!("Expected id in forward but was: {:?}", other),
                    (other, _) =>
                        panic!("Expected forward on an id, but tried to forward on: {:?}", other),
                });

                Core::Block { statements }
            }
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

fn forward_def(id: String, method: String) -> Core {
    // TODO derive args from object type and method from context
    let args = vec![Core::Id { lit: String::from("self") }];
    let object = Box::from(Core::PropertyCall {
        object:   Box::from(Core::Id { lit: String::from("self") }),
        property: id
    });

    Core::FunDef {
        private: false,
        id:      Box::from(Core::Id { lit: method.clone() }),
        args:    args.clone(),
        body:    Box::from(Core::MethodCall { object, method, args })
    }
}
