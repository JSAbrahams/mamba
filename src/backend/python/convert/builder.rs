use crate::backend::python::ast::node::PythonCore;
use crate::backend::python::convert::common::convert_vec;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::result::{GenResult, UnimplementedErr};
use crate::check::ast::NodeTy;
use crate::{ASTTy, Context};

pub fn convert_builder(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    match &ast.node {
        NodeTy::DictBuilder {
            from,
            to,
            conditions,
        } => {
            let from = Box::from(convert_node(from, imp, state, ctx)?);
            let to = Box::from(convert_node(to, imp, state, ctx)?);

            if let Some(col) = conditions.first() {
                let conds = conditions
                    .strip_prefix(std::slice::from_ref(col))
                    .expect("Unreachable");
                let conds = convert_vec(conds, imp, state, ctx)?;
                let col = Box::from(convert_node(col, imp, state, ctx)?);
                Ok(PythonCore::DictComprehension {
                    from,
                    to,
                    col,
                    conds,
                })
            } else {
                Err(Box::from(UnimplementedErr::new(ast, "Cannot be empty")))
            }
        }
        NodeTy::ListBuilder { item, conditions } => {
            let expr = Box::from(convert_node(item, imp, state, ctx)?);

            if let Some(col) = conditions.first() {
                let conds = conditions
                    .strip_prefix(std::slice::from_ref(col))
                    .expect("Unreachable");
                let conds = convert_vec(conds, imp, state, ctx)?;
                let col = Box::from(convert_node(col, imp, state, ctx)?);
                Ok(PythonCore::List {
                    elements: vec![PythonCore::Comprehension { expr, col, conds }],
                })
            } else {
                Err(Box::from(UnimplementedErr::new(ast, "Cannot be empty")))
            }
        }
        NodeTy::SetBuilder { item, conditions } => {
            let expr = Box::from(convert_node(item, imp, state, ctx)?);

            if let Some(col) = conditions.first() {
                let conds = conditions
                    .strip_prefix(std::slice::from_ref(col))
                    .expect("Unreachable");
                let conds = convert_vec(conds, imp, state, ctx)?;
                let col = Box::from(convert_node(col, imp, state, ctx)?);
                Ok(PythonCore::Set {
                    elements: vec![PythonCore::Comprehension { expr, col, conds }],
                })
            } else {
                Err(Box::from(UnimplementedErr::new(ast, "Cannot be empty")))
            }
        }
        other => {
            let msg = format!("Expected call flow but was: {other:?}.");
            Err(Box::from(UnimplementedErr::new(ast, &msg)))
        }
    }
}
