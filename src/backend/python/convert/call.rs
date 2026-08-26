use crate::backend::python::ast::node::PythonCore;
use crate::backend::python::convert::common::convert_vec;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::name::ToPy;
use crate::backend::python::result::{GenResult, UnimplementedErr};
use crate::check::ast::NodeTy;
use crate::{ASTTy, Context};

pub fn convert_call(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    Ok(match &ast.node {
        NodeTy::PropertyCall { instance, property } => PythonCore::PropertyCall {
            object: Box::from(convert_node(instance, imp, state, ctx)?),
            property: Box::from(convert_node(property, imp, state, ctx)?),
        },
        NodeTy::FunctionCall { name, args } => PythonCore::FunctionCall {
            function: Box::from(name.to_py(imp)),
            args: convert_vec(args, imp, state, ctx)?,
        },
        other => {
            let msg = format!("Expected call flow but was: {other:?}.");
            return Err(Box::from(UnimplementedErr::new(ast, &msg)));
        }
    })
}
