use crate::backend::python::ast::node::PythonCore;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::result::{GenResult, UnimplementedErr};
use crate::check::ast::NodeTy;
use crate::check::context::clss;
use crate::{ASTTy, Context};

pub fn convert_range(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    let NodeTy::Range {
        from,
        to,
        inclusive,
        step,
    } = &ast.node
    else {
        let msg = format!("Expected range, was {:?}", ast.node);
        return Err(Box::from(UnimplementedErr::new(ast, &msg)));
    };

    // Python's `range` excludes its upper bound, so an inclusive one is that bound plus one.
    Ok(PythonCore::FunctionCall {
        function: Box::from(PythonCore::Id {
            lit: String::from(clss::python::RANGE),
        }),
        args: vec![
            convert_node(from, imp, state, ctx)?,
            if *inclusive {
                PythonCore::Add {
                    left: Box::from(convert_node(to, imp, state, ctx)?),
                    right: Box::from(PythonCore::Int {
                        int: String::from("1"),
                    }),
                }
            } else {
                convert_node(to, imp, state, ctx)?
            },
            if let Some(step) = step {
                convert_node(step, imp, state, ctx)?
            } else {
                PythonCore::Int {
                    int: String::from("1"),
                }
            },
        ],
    })
}
