use crate::parser::ast::ASTNodePos;
use crate::type_checker::context::class::get_type;
use crate::type_checker::context::field::{get_fields, Field};
use crate::type_checker::context::function::{get_functions, Function};
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::TypeResult;

pub mod class;
pub mod field;
pub mod function;

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
#[derive(Debug)]
pub struct Context {
    types:     Vec<Type>,
    functions: Vec<Function>,
    fields:    Vec<Field>
}

pub fn build_context(ast_trees: &[ASTNodePos]) -> TypeResult<Context> {
    let mut types = vec![];
    let mut functions = vec![];
    let mut fields = vec![];

    for ast_tree_file in ast_trees {
        types.append(&mut get_type(ast_tree_file)?);
        functions.append(&mut get_functions(ast_tree_file)?);
        fields.append(&mut get_fields(ast_tree_file)?);
    }

    Ok(Context { types, functions, fields })
}
