use crate::parser::ast::ASTNodePos;
use crate::type_checker::context::class::get_classes;
use crate::type_checker::context::field::{Field, get_fields};
use crate::type_checker::context::function::{Function, get_functions};
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::{TypeResult};

pub mod class;
pub mod field;
pub mod function;

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
pub struct Context {
    classes: Vec<Type>,
    functions: Vec<Function>,
    fields: Vec<Field>,
}

pub fn build_context(ast_trees: &[ASTNodePos]) -> TypeResult<Context> {
    let mut classes = vec![];
    let mut functions = vec![];
    let mut fields = vec![];

    for ast_tree in ast_trees {
        classes.append(&mut get_classes(ast_tree)?);
        functions.append(&mut get_functions(ast_tree)?);
        fields.append(&mut get_fields(ast_tree)?);
    }

    Ok(Context { classes, functions, fields })
}
