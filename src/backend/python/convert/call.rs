use crate::backend::python::ast::node::PythonCore;
use crate::backend::python::convert::common::convert_vec;
use crate::backend::python::convert::convert_node;
use crate::backend::python::convert::state::{Imports, State};
use crate::backend::python::name::ToPy;
use crate::backend::python::result::{GenResult, UnimplementedErr};
use crate::check::ast::NodeTy;
use crate::check::context::clss::GetFun;
use crate::check::context::LookupClass;
use crate::check::name::string_name::StringName;
use crate::check::NEW;
use crate::common::position::Position;
use crate::{ASTTy, Context};

/// Rewrite `C.new(..)` to `C(..)` where `new` is the constructor the class got for free.
///
/// [None] when this is any other property call, including a `new` the class declares itself.
fn generated_new_as_construction(
    instance: &ASTTy,
    property: &ASTTy,
    ctx: &Context,
) -> Option<ASTTy> {
    let class = match &instance.node {
        NodeTy::Id { lit } => StringName::from(lit.as_str()),
        _ => return None,
    };
    let (name, args) = match &property.node {
        NodeTy::FunctionCall { name, args, .. } if name.name.as_str() == NEW => (name, args),
        _ => return None,
    };

    let class = ctx.class(&class, Position::invisible()).ok()?;
    if class.fun(name, Position::invisible()).is_ok() {
        // Declared its own new, which is a real method.
        return None;
    }

    Some(ASTTy {
        pos: property.pos,
        ty: property.ty.clone(),
        node: NodeTy::FunctionCall {
            name: class.name,
            args: args.clone(),
            is_index: false,
        },
    })
}

pub fn convert_call(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    Ok(match &ast.node {
        NodeTy::PropertyCall { instance, property } => {
            // A class's generated `new` is sugar for applying the class to its arguments,
            // so it lowers to the plain construction Python already has. A class that
            // declares its own `new` has a real method, and is left alone.
            if let Some(call) = generated_new_as_construction(instance, property, ctx) {
                convert_node(&call, imp, state, ctx)?
            } else {
                PythonCore::PropertyCall {
                    object: Box::from(convert_node(instance, imp, state, ctx)?),
                    property: Box::from(convert_node(property, imp, state, ctx)?),
                }
            }
        }
        NodeTy::FunctionCall {
            name,
            is_index: true,
            args,
        } => PythonCore::Index {
            item: Box::from(name.to_py(imp)),
            range: Box::from(convert_node(&args[0], imp, state, ctx)?),
        },
        NodeTy::FunctionCall { name, args, .. } => PythonCore::FunctionCall {
            function: Box::from(name.to_py(imp)),
            args: convert_vec(args, imp, state, ctx)?,
        },
        other => {
            let msg = format!("Expected call flow but was: {other:?}.");
            return Err(Box::from(UnimplementedErr::new(ast, &msg)));
        }
    })
}
