use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Imports;
use crate::desugar::context::State;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::parser::ast::Node;
use crate::parser::ast::AST;

pub fn desugar_definition(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    Ok(match &ast.node {
        Node::VariableDef { id_maybe_type, expression, private, .. } => {
            let id = desugar_node(id_maybe_type, imp, state)?;
            let new_state = state.in_tup(match id.clone() {
                Core::Tuple { elements } => elements.len(),
                _ => 1
            });

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
        Node::FunDef { id, fun_args, body: expression, private, .. } => Core::FunDef {
            private: *private,
            id:      Box::from(desugar_node(&id, imp, state)?),
            args:    desugar_vec(&fun_args, imp, state)?,
            body:    if state.interface {
                Box::from(Core::Pass)
            } else {
                Box::from(match expression {
                    Some(expr) => desugar_node(&expr, imp, &state.expand_ty(true))?,
                    None => Core::Empty
                })
            }
        },
        definition => panic!("Expected definition: {:?}.", definition)
    })
}
