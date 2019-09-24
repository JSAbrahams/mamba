use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

use python_parser::ast::{Classdef, CompoundStatement, Statement};

use crate::common::position::Position;
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::parent::GenericParent;
use crate::type_checker::context::generic::GenericType;
use crate::type_checker::context::python::field::GenericFields;
use crate::type_checker::context::python::function::INIT;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub mod field;
pub mod function;

mod function_arg;
mod parent;
pub mod type_name;

pub fn python_files() -> TypeResult<(Vec<GenericType>, Vec<GenericField>, Vec<GenericFunction>)> {
    let mut types = vec![];
    let mut fields = vec![];
    let mut functions = vec![];

    let python_primitives = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("type_checker")
        .join("resources")
        .join("primitives");
    let entries = fs::read_dir(python_primitives)
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

impl From<&Classdef> for GenericType {
    fn from(class_def: &Classdef) -> GenericType {
        let mut functions = vec![];
        let mut fields = vec![];
        for statement in &class_def.code {
            match statement {
                Statement::Assignment(variables, expressions) =>
                    fields.append(&mut GenericFields::from((variables, expressions)).fields),
                Statement::TypedAssignment(variables, ty, expressions) =>
                    fields.append(&mut GenericFields::from((variables, expressions)).fields),
                Statement::Compound(compound) => match compound.deref() {
                    CompoundStatement::Funcdef(func_def) =>
                        functions.push(GenericFunction::from(func_def)),
                    _ => {}
                },
                _ => {}
            }
        }

        let args = functions
            .iter()
            .find(|f| f.name.as_str() == INIT)
            .map_or(vec![], |f| f.arguments.clone());

        GenericType {
            is_py_type: true,
            name: class_def.name.clone(),
            pos: Position::default(),
            concrete: false,
            args,
            generics: vec![],
            fields,
            functions,
            parents: class_def.arguments.iter().map(GenericParent::from).collect()
        }
    }
}
