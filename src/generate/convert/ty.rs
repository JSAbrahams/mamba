use crate::ASTTy;
use crate::check::ast::NodeTy;
use crate::check::context::clss::concrete_to_python;
use crate::generate::ast::node::Core;
use crate::generate::convert::common::convert_vec;
use crate::generate::convert::convert_node;
use crate::generate::convert::state::{Imports, State};
use crate::generate::result::GenResult;

pub fn convert_ty(ast: &ASTTy, imp: &mut Imports, state: &State) -> GenResult {
    Ok(match &ast.node {
        NodeTy::QuestionOp { expr } => {
            imp.add_from_import("typing", "Optional");
            Core::Type {
                lit: String::from("Optional"),
                generics: vec![convert_node(expr, imp, state)?],
            }
        }
        NodeTy::Id { lit } => Core::Id { lit: concrete_to_python(lit) },
        NodeTy::ExpressionType { expr, .. } => convert_node(expr, imp, state)?,
        NodeTy::TypeTup { types } => {
            imp.add_from_import("typing", "Tuple");
            Core::Type {
                lit: String::from("Tuple"),
                generics: convert_vec(types, imp, state)?,
            }
        }
        NodeTy::Type { id, generics } => match &id.node {
            NodeTy::Id { lit } => Core::Type {
                lit: concrete_to_python(lit),
                generics: convert_vec(generics, imp, state)?,
            },
            other => panic!("Expected identifier but was {:?}", other)
        },
        NodeTy::TypeFun { args, ret_ty } => {
            imp.add_from_import("typing", "Callable");
            Core::Type {
                lit: String::from("Callable"),
                generics: vec![
                    Core::List { elements: convert_vec(args, imp, state)? },
                    convert_node(ret_ty, imp, state)?,
                ],
            }
        }
        NodeTy::TypeUnion { types } => {
            imp.add_from_import("typing", "Union");
            Core::Type {
                lit: String::from("Union"),
                generics: convert_vec(types, imp, state)?,
            }
        }
        ty => panic!("Expected type: {:?}.", ty)
    })
}
