use crate::{ASTTy, Context};
use crate::check::ast::NodeTy;
use crate::generate::ast::node::Core;
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, UnimplementedErr};

pub fn convert_call(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    Ok(match &ast.node {
        NodeTy::PropertyCall { instance, property } => Core::PropertyCall {
            object: Box::from(convert_node(instance, imp, state, ctx)?),
            property: Box::from(convert_node(property, imp, state, ctx)?),
        },
        NodeTy::FunctionCall { name, args } => Core::FunctionCall {
            function: Box::from(convert_node(name, imp, state, ctx)?),
            args: convert_vec(args, imp, state, ctx)?,
        },
        other => {
            let msg = format!("Expected call flow but was: {:?}.", other);
            return Err(Box::from(UnimplementedErr::new(ast, &msg)));
        }
    })
}
