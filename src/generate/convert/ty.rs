use crate::{ASTTy, Context};
use crate::check::ast::NodeTy;
use crate::check::context::clss::concrete_to_python;
use crate::check::context::clss::python::{CALLABLE, UNION};
use crate::generate::ast::node::Core;
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::{GenResult, UnimplementedErr};

pub fn convert_ty(ast: &ASTTy, imp: &mut Imports, state: &State, ctx: &Context) -> GenResult {
    Ok(match &ast.node {
        NodeTy::QuestionOp { expr } => {
            imp.add_from_import("typing", "Optional");
            Core::Type {
                lit: String::from("Optional"),
                generics: vec![convert_node(expr, imp, state, ctx)?],
            }
        }
        NodeTy::Id { lit } => Core::Id { lit: concrete_to_python(lit) },
        NodeTy::ExpressionType { expr, .. } => convert_node(expr, imp, state, ctx)?,
        NodeTy::TypeTup { types } => {
            imp.add_from_import("typing", "Tuple");
            Core::Type {
                lit: String::from("Tuple"),
                generics: convert_vec(types, imp, state, ctx)?,
            }
        }
        NodeTy::Type { id, generics } => match &id.node {
            NodeTy::Id { lit } => Core::Type {
                lit: concrete_to_python(lit),
                generics: convert_vec(generics, imp, state, ctx)?,
            },
            other => {
                let msg = format!("Expected identifier but was {other:?}");
                return Err(Box::from(UnimplementedErr::new(ast, &msg)));
            }
        },
        NodeTy::TypeFun { args, ret_ty } => {
            imp.add_from_import("typing", CALLABLE);
            Core::Type {
                lit: String::from("Callable"),
                generics: vec![
                    Core::List { elements: convert_vec(args, imp, state, ctx)? },
                    convert_node(ret_ty, imp, state, ctx)?,
                ],
            }
        }
        NodeTy::TypeUnion { types } => {
            imp.add_from_import("typing", UNION);
            Core::Type {
                lit: String::from(UNION),
                generics: convert_vec(types, imp, state, ctx)?,
            }
        }
        ty => {
            let msg = format!("Expected type: {ty:?}.");
            return Err(Box::from(UnimplementedErr::new(ast, &msg)));
        }
    })
}
