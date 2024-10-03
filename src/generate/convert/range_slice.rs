use crate::check::ast::NodeTy;
use crate::check::context::clss;
use crate::generate::ast::node::Core;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, UnimplementedErr};
use crate::{ASTTy, Context};

pub fn convert_range_slice(
    ast: &ASTTy,
    imp: &mut Imports,
    state: &State,
    ctx: &Context,
) -> GenResult {
    match &ast.node {
        NodeTy::Range { from, to, inclusive, step } => Ok(Core::FunctionCall {
            function: Box::from(Core::Id { lit: String::from(clss::python::RANGE) }),
            args: vec![
                convert_node(from, imp, state, ctx)?,
                if *inclusive {
                    Core::Add {
                        left: Box::from(convert_node(to, imp, state, ctx)?),
                        right: Box::from(Core::Int { int: String::from("1") }),
                    }
                } else {
                    convert_node(to, imp, state, ctx)?
                },
                if let Some(step) = step {
                    convert_node(step, imp, state, ctx)?
                } else {
                    Core::Int { int: String::from("1") }
                },
            ],
        }),
        NodeTy::Slice { from, to, inclusive, step } => Ok(Core::FunctionCall {
            function: Box::from(Core::Id { lit: String::from(clss::python::SLICE) }),
            args: vec![
                convert_node(from, imp, state, ctx)?,
                if !inclusive {
                    Core::Sub {
                        left: Box::from(convert_node(to, imp, state, ctx)?),
                        right: Box::from(Core::Int { int: String::from("1") }),
                    }
                } else {
                    convert_node(to, imp, state, ctx)?
                },
                if let Some(step) = step {
                    convert_node(step, imp, state, ctx)?
                } else {
                    Core::Int { int: String::from("1") }
                },
            ],
        }),
        other => {
            let msg = format!("Expected range or slice, was {other:?}");
            Err(Box::from(UnimplementedErr::new(ast, &msg)))
        }
    }
}
