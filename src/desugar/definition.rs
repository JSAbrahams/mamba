use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Imports;
use crate::desugar::context::State;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;

pub fn desugar_definition(
    node_pos: &ASTNodePos,
    imp: &mut Imports,
    state: &State
) -> DesugarResult {
    Ok(match &node_pos.node {
        ASTNode::VariableDef { id_maybe_type, expression, private, .. } => {
            let id = desugar_node(id_maybe_type, imp, state)?;
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
                    (_, Some(expr)) => Box::from(desugar_node(&expr, imp, &new_state)?),
                    (Core::Tuple { elements }, None) =>
                        Box::from(Core::Tuple { elements: vec![Core::None; elements.len()] }),
                    (_, None) => Box::from(Core::None)
                }
            }
        }
        ASTNode::FunDef { id, fun_args, body: expression, private, .. } => Core::FunDef {
            private: *private,
            id:      Box::from(desugar_node(&id, imp, state)?),
            args:    desugar_vec(&fun_args, imp, state)?,
            body:    if state.interface {
                Box::from(Core::Pass)
            } else {
                Box::from(match expression {
                    Some(expr) => desugar_node(&expr, imp, &State {
                        tup:         state.tup,
                        expect_expr: true,
                        interface:   state.interface
                    })?,
                    None => Core::Empty
                })
            }
        },
        definition => panic!("Expected definition: {:?}.", definition)
    })
}
