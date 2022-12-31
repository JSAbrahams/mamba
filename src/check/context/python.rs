use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;

use python_parser::ast::{CompoundStatement, Statement};

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::context::function::generic::GenericFunction;
use crate::check::result::{TypeErr, TypeResult};

pub fn python_files(
    python_dir: &Path
) -> TypeResult<(HashSet<GenericClass>, HashSet<GenericField>, HashSet<GenericFunction>)> {
    let mut types = HashSet::new();
    let (mut fields, mut functions) = (HashSet::new(), HashSet::new());

    let entries = fs::read_dir(python_dir)
        .map_err(|io_err| TypeErr::new_no_pos(io_err.to_string().as_str()))?;

    for entry in entries {
        let path = entry.map_err(|err| TypeErr::new_no_pos(err.to_string().as_str()))?.path();
        let python_src_path = path
            .as_os_str()
            .to_str()
            .ok_or_else(|| TypeErr::new_no_pos("Unable to build context for python resource"))?;

        let mut python_src = String::new();
        match File::open(python_src_path) {
            Ok(mut path) => path.read_to_string(&mut python_src).map_err(|err| {
                TypeErr::new_no_pos(format!("Unable to read python file: {:?}", err).as_str())
            })?,
            Err(_) => return Err(vec![TypeErr::new_no_pos("primitive does not exist")])
        };

        let python_src = python_src.replace("\r\n", "\n"); // Replace CRLF
        let statements =
            python_parser::file_input(python_parser::make_strspan(python_src.as_ref())).unwrap().1;

        for statement in statements {
            match &statement {
                Statement::Assignment(left, _) =>
                    GenericFields::from((left, &None)).fields.into_iter().for_each(|field| {
                        fields.insert(field);
                    }),
                Statement::TypedAssignment(left, ty, _) =>
                    GenericFields::from((left, &Some(ty.clone()))).fields.into_iter().for_each(|field| {
                        fields.insert(field);
                    }),
                Statement::Compound(compound_stmt) => {
                    match compound_stmt.deref() {
                        CompoundStatement::Funcdef(func_def) =>
                            functions.insert(GenericFunction::from(func_def)),
                        CompoundStatement::Classdef(class_def) =>
                            types.insert(GenericClass::try_from(class_def)?),
                        _ => { false }
                    };
                }
                _ => {}
            }
        }
    }

    Ok((types, fields, functions))
}
