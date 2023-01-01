use crate::{ASTTy, Context};
use crate::check::ast::NodeTy;
use crate::generate::ast::node::Core;
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, UnimplementedErr};

pub fn convert_builder(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    match &ast.node {
        NodeTy::ListBuilder { item, conditions } => {
            let expr = Box::from(convert_node(item, imp, state, ctx)?);

            if let Some(col) = conditions.first() {
                let conds = conditions.strip_prefix(&[col.clone()]).expect("Unreachable");
                let conds = convert_vec(conds, imp, state, ctx)?;
                let col = Box::from(convert_node(col, imp, state, ctx)?);
                Ok(Core::List { elements: vec![Core::Comprehension { expr, col, conds }] })
            } else {
                Err(Box::from(UnimplementedErr::new(ast, "Cannot be empty")))
            }
        }
        NodeTy::SetBuilder { item, conditions } => {
            let expr = Box::from(convert_node(item, imp, state, ctx)?);

            if let Some(col) = conditions.first() {
                let conds = conditions.strip_prefix(&[col.clone()]).expect("Unreachable");
                let conds = convert_vec(conds, imp, state, ctx)?;
                let col = Box::from(convert_node(col, imp, state, ctx)?);
                Ok(Core::Set { elements: vec![Core::Comprehension { expr, col, conds }] })
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
