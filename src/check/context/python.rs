use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::PathBuf;

use python_parser::ast::{CompoundStatement, Statement};

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::context::function::generic::GenericFunction;
use crate::check::result::{TypeErr, TypeResult};

pub fn python_files(
    python_dir: &PathBuf
) -> TypeResult<(HashSet<GenericClass>, HashSet<GenericField>, HashSet<GenericFunction>)> {
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
                        types.push(GenericClass::try_from(class_def)?),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    Ok((
        types.into_iter().collect::<HashSet<_>>(),
        fields.into_iter().collect::<HashSet<_>>(),
        functions.into_iter().collect::<HashSet<_>>()
    ))
}
