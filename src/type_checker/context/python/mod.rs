use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

use python_parser::ast::{CompoundStatement, Statement};

use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::ty::GenericType;
use crate::type_checker::context::python::field::GenericFields;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub mod field;
pub mod function;
pub mod ty;

mod function_arg;
mod parent;
pub mod type_name;

pub fn python_files(
    python_dir: &PathBuf
) -> TypeResult<(Vec<GenericType>, Vec<GenericField>, Vec<GenericFunction>)> {
    let mut types = vec![];
    let mut fields = vec![];
    let mut functions = vec![];

    let entries = fs::read_dir(python_dir)
        .map_err(|io_err| TypeErr::new_no_pos(io_err.to_string().as_str()))?;

    for entry in entries {
        let path = entry.map_err(|err| TypeErr::new_no_pos(err.to_string().as_str()))?.path();

        let python_src = path
            .as_os_str()
            .to_str()
            .ok_or_else(|| TypeErr::new_no_pos("Unable to build context for primitive"))?;
        let statements =
            python_parser::file_input(python_parser::make_strspan(python_src.as_ref())).unwrap().1;

        for statement in statements {
            match &statement {
                Statement::Assignment(left, right) =>
                    fields.append(&mut GenericFields::from((left, right)).fields),
                Statement::TypedAssignment(left, ty, right) =>
                    fields.append(&mut GenericFields::from((left, right)).fields),
                Statement::Compound(compound_stmt) => match compound_stmt.deref() {
                    CompoundStatement::Funcdef(func_def) =>
                        functions.push(GenericFunction::from(func_def)),
                    CompoundStatement::Classdef(class_def) =>
                        types.push(GenericType::from(class_def)),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok((types, fields, functions))
}
