use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::parser::ast::Node;
use crate::parser::ast::AST;

pub fn desugar_definition(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    // TODO augment function definition in type checker so that it has return type
    // when applicable
    Ok(match &ast.node {
        Node::VariableDef { id_maybe_type, expression, private, .. } => {
            let id = desugar_node(id_maybe_type, imp, state)?;
            let state = state.in_tup(match id.clone() {
                Core::Tuple { elements } => elements.len(),
                _ => 1
            });

            Core::VarDef {
                private: *private,
                id:      Box::from(id.clone()),
                right:   match (id.clone(), expression) {
                    (_, Some(expr)) => Box::from(desugar_node(&expr, imp, &state)?),
                    (Core::Tuple { elements }, None) =>
                        Box::from(Core::Tuple { elements: vec![Core::None; elements.len()] }),
                    (_, None) => Box::from(Core::None)
                }
            }
        }
        Node::FunDef { id, fun_args, body: expression, private, ret_ty, .. } => Core::FunDef {
            private: *private,
            id:      Box::from(desugar_node(&id, imp, state)?),
            args:    desugar_vec(&fun_args, imp, state)?,
            ret_ty:  match ret_ty {
                Some(ret_ty) => Some(Box::from(desugar_node(ret_ty, imp, state)?)),
                None => None
            },
            body:    if state.interface {
                Box::from(Core::Pass)
            } else {
                // TODO augment AST in type checker
                Box::from(match expression {
                    Some(expr) => desugar_node(&expr, imp, &state.expand_ty(true))?,
                    None => Core::Empty
                })
            }
        },
        definition => panic!("Expected definition: {:?}.", definition)
    })
}
