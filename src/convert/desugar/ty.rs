use crate::check::context::clss::concrete_to_python;
use crate::convert::ast::node::Core;
use crate::convert::desugar::common::desugar_vec;
use crate::convert::desugar::desugar_node;
use crate::convert::desugar::state::{Imports, State};
use crate::convert::result::ConvertResult;
use crate::parse::ast::{AST, Node};

pub fn desugar_type(ast: &AST, imp: &mut Imports, state: &State) -> ConvertResult {
    Ok(match &ast.node {
        Node::QuestionOp { expr } => {
            imp.add_from_import("typing", "Optional");
            Core::Type {
                lit: String::from("Optional"),
                generics: vec![desugar_node(expr, imp, state)?],
            }
        }
        Node::Id { lit } => Core::Id { lit: concrete_to_python(lit) },
        Node::ExpressionType { expr, .. } => desugar_node(expr, imp, state)?,
        Node::TypeTup { types } => {
            imp.add_from_import("typing", "Tuple");
            Core::Type {
                lit: String::from("Tuple"),
                generics: desugar_vec(types, imp, state)?,
            }
        }
        Node::Type { id, generics } => match &id.node {
            Node::Id { lit } => Core::Type {
                lit: concrete_to_python(lit),
                generics: desugar_vec(generics, imp, state)?,
            },
            other => panic!("Expected identifier but was {:?}", other)
        },
        Node::TypeFun { args, ret_ty } => {
            imp.add_from_import("typing", "Callable");
            Core::Type {
                lit: String::from("Callable"),
                generics: vec![
                    Core::List { elements: desugar_vec(args, imp, state)? },
                    desugar_node(ret_ty, imp, state)?,
                ],
            }
        }
        Node::TypeUnion { types } => {
            imp.add_from_import("typing", "Union");
            Core::Type {
                lit: String::from("Union"),
                generics: desugar_vec(types, imp, state)?,
            }
        }
        ty => panic!("Expected type: {:?}.", ty)
    })
}
