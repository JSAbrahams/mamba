use crate::check::context::clss::concrete_to_python;
use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::node::desugar_node;
use crate::desugar::result::DesugarResult;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::parse::ast::Node;
use crate::parse::ast::AST;

pub fn desugar_type(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    Ok(match &ast.node {
        Node::QuestionOp { expr } => {
            imp.add_from_import("typing", "Optional");
            Core::Type {
                lit:      String::from("Optional"),
                generics: vec![desugar_node(expr, imp, state)?]
            }
        }
        Node::Id { lit } => Core::Id { lit: concrete_to_python(lit) },
        Node::ExpressionType { expr, .. } => desugar_node(expr, imp, state)?,
        Node::TypeTup { types } => {
            imp.add_from_import("typing", "Tuple");
            Core::Type {
                lit:      String::from("Tuple"),
                generics: desugar_vec(types, imp, state)?
            }
        }
        Node::Type { id, generics } => match &id.node {
            Node::Id { lit } => Core::Type {
                lit:      concrete_to_python(&lit),
                generics: desugar_vec(generics, imp, state)?
            },
            other => panic!("Expected identifier but was {:?}", other)
        },
        Node::TypeFun { args, ret_ty } => {
            imp.add_from_import("typing", "Callable");
            Core::Type {
                lit:      String::from("Callable"),
                generics: vec![
                    Core::List { elements: desugar_vec(args, imp, state)? },
                    desugar_node(ret_ty, imp, state)?,
                ]
            }
        }
        Node::TypeUnion { types } => {
            imp.add_from_import("typing", "Union");
            Core::Type {
                lit:      String::from("Union"),
                generics: desugar_vec(types, imp, state)?
            }
        }
        ty => panic!("Expected type: {:?}.", ty)
    })
}
