use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;

use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::field::python::GenericFields;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use python_parser::ast::{CompoundStatement, Statement};
use std::collections::HashSet;
use std::iter::FromIterator;

pub fn python_files(
    python_dir: &PathBuf
) -> TypeResult<(HashSet<GenericType>, HashSet<GenericField>, HashSet<GenericFunction>)> {
    let mut types = vec![];
    let mut fields = vec![];
    let mut functions = vec![];

    let entries = fs::read_dir(python_dir)
        .map_err(|io_err| TypeErr::new_no_pos(io_err.to_string().as_str()))?;

    for entry in entries {
        let path = entry.map_err(|err| TypeErr::new_no_pos(err.to_string().as_str()))?.path();
        let python_src_path = path
            .as_os_str()
            .to_str()
            .ok_or_else(|| TypeErr::new_no_pos("Unable to build context for primitive"))?;

        let mut python_src = String::new();
        match File::open(python_src_path) {
            Ok(mut path) => path.read_to_string(&mut python_src).map_err(|err| {
                TypeErr::new_no_pos(format!("Unable to read python file: {:?}", err).as_str())
            })?,
            Err(_) => return Err(vec![TypeErr::new_no_pos("primitive does not exist")])
        };
        let statements =
            python_parser::file_input(python_parser::make_strspan(python_src.as_ref())).unwrap().1;

        for statement in statements {
            match &statement {
                Statement::Assignment(left, right) => fields
                    .append(&mut GenericFields::from((left, right)).fields.into_iter().collect()),
                // TODO use type hints
                Statement::TypedAssignment(left, _, right) => fields
                    .append(&mut GenericFields::from((left, right)).fields.into_iter().collect()),
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

    Ok((
        HashSet::from_iter(types.into_iter()),
        HashSet::from_iter(fields.into_iter()),
        HashSet::from_iter(functions.into_iter())
    ))
}
