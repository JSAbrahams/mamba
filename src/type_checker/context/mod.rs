use crate::parser::ast::ASTNodePos;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::TypeResult;

pub mod class;
pub mod field;
pub mod function;
pub mod type_name;

mod common;

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

pub fn build_context(_: &[ASTNodePos]) -> TypeResult<Context> {
    let types = vec![];
    let functions = vec![];
    let fields = vec![];

    Ok(Context { types, functions, fields })
}
