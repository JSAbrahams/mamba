use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::node::desugar_node;
use crate::desugar::result::DesugarResult;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::parse::ast::AST;
use crate::parse::ast::Node;

pub fn desugar_definition(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    // TODO augment function definition in type checker so that it has return type
    // when applicable
    Ok(match &ast.node {
        Node::VariableDef { var, expr: expression, ty, .. } => {
            let var = desugar_node(var, imp, state)?;
            let state = state.in_tup(match var.clone() {
                Core::Tuple { elements } => elements.len(),
                _ => 1
            });

            Core::VarDef {
                var: Box::from(var.clone()),
                ty: match ty {
                    Some(ty) => Some(Box::from(desugar_node(ty, imp, &state)?)),
                    None => None
                },
                expr: match (var, expression) {
                    (_, Some(expr)) => Some(Box::from(desugar_node(expr, imp, &state)?)),
                    (Core::Tuple { elements }, None) =>
                        Some(Box::from(Core::Tuple { elements: vec![Core::None; elements.len()] })),
                    (_, None) => None
                },
            }
        }
        Node::FunDef { id, args: fun_args, body: expression, ret: ret_ty, .. } => Core::FunDef {
            id: Box::from(desugar_node(id, imp, state)?),
            arg: desugar_vec(fun_args, imp, state)?,
            ty: match ret_ty {
                Some(ret_ty) => Some(Box::from(desugar_node(ret_ty, imp, state)?)),
                None => None
            },
            body: if state.interface {
                Box::from(Core::Pass)
            } else {
                // TODO augment AST in type checker
                Box::from(match expression {
                    Some(expr) => desugar_node(expr, imp, &state.expand_ty(true))?,
                    None => Core::Empty
                })
            },
        },
        definition => panic!("Expected definition: {:?}.", definition)
    })
}
