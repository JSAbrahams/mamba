use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;

pub fn desugar_definition(node: &ASTNode, ctx: &mut Context, state: &State) -> DesugarResult {
    Ok(match node {
        ASTNode::Def { private, definition, .. } => match &definition.node {
            ASTNode::VariableDef { id_maybe_type, expression, .. } => {
                let id = desugar_node(id_maybe_type, ctx, state)?;
                let new_state = &State {
                    tup:         match id.clone() {
                        Core::Tuple { elements } => elements.len(),
                        _ => 1
                    },
                    expect_expr: true,
                    interface:   state.interface
                };

                Core::VarDef {
                    private: *private,
                    id:      Box::from(id.clone()),
                    right:   match (id.clone(), expression) {
                        (_, Some(expr)) => Box::from(desugar_node(&expr, ctx, &new_state)?),
                        (Core::Tuple { elements }, None) =>
                            Box::from(Core::Tuple { elements: vec![Core::None; elements.len()] }),
                        (_, None) => Box::from(Core::None)
                    }
                }
            }
            ASTNode::FunDef { id, fun_args, body: expression, .. } => Core::FunDef {
                private: *private,
                id:      Box::from(desugar_node(&id, ctx, state)?),
                args:    desugar_vec(&fun_args, ctx, state)?,
                body:    if state.interface {
                    Box::from(Core::Pass)
                } else {
                    Box::from(match expression {
                        Some(expr) => desugar_node(&expr, ctx, &State {
                            tup:         state.tup,
                            expect_expr: true,
                            interface:   state.interface
                        })?,
                        None => Core::Empty
                    })
                }
            },
            definition => panic!("invalid definition format: {:?}.", definition)
        },
        other => panic!("Expected definition but was: {:?}.", other)
    })
}
