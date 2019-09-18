use std::convert::TryFrom;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

use python_parser::ast::{Classdef, CompoundStatement, Statement};

use crate::common::position::Position;
use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::parent::GenericParent;
use crate::type_checker::context::generic::GenericType;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub mod field;
pub mod function;

mod function_arg;
mod parent;
mod type_name;

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
                    fields.push(GenericField::try_from((left, right))),
                Statement::TypedAssignment(left, ty, right) =>
                    fields.push(GenericField::try_from((left, ty, right))),
                Statement::Compound(compound_stmt) => match compound_stmt.deref() {
                    CompoundStatement::Funcdef(func_def) =>
                        functions.push(GenericFunction::try_from(func_def)),
                    CompoundStatement::Classdef(class_def) =>
                        types.push(GenericType::try_from(class_def)),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    let (types, ty_errs): (Vec<_>, Vec<_>) = types.into_iter().partition(Result::is_ok);
    let (fields, field_errs): (Vec<_>, Vec<_>) = fields.into_iter().partition(Result::is_ok);
    let (functions, fun_errs): (Vec<_>, Vec<_>) = functions.into_iter().partition(Result::is_ok);

    if ty_errs.is_empty() && fun_errs.is_empty() && fun_errs.is_empty() {
        Ok((
            types.into_iter().map(Result::unwrap).collect(),
            fields.into_iter().map(Result::unwrap).collect(),
            functions.into_iter().map(Result::unwrap).collect()
        ))
    } else {
        let ty_errs: Vec<_> = ty_errs.into_iter().map(Result::unwrap_err).collect();
        let field_errs: Vec<_> = field_errs.into_iter().map(Result::unwrap_err).collect();
        let fun_errs: Vec<_> = fun_errs.into_iter().map(Result::unwrap_err).collect();
        Err(vec![ty_errs, field_errs, fun_errs].into_iter().flatten().flatten().collect())
    }
}

impl TryFrom<&Classdef> for GenericType {
    type Error = Vec<TypeErr>;

    fn try_from(class_def: &Classdef) -> TypeResult<GenericType> {
        let mut name = String::new();
        let mut parents: Vec<Result<GenericParent, Vec<TypeErr>>> = vec![];
        let mut functions: Vec<Result<GenericFunction, Vec<TypeErr>>> = vec![];
        let mut fields: Vec<Result<GenericField, Vec<TypeErr>>> = vec![];

        name = class_def.name.clone();

        for argument in &class_def.arguments {
            parents.push(GenericParent::try_from(argument))
        }

        for statement in &class_def.code {
            match statement {
                Statement::Assignment(variables, expressions) =>
                    fields.push(GenericField::try_from((variables, expressions))),
                Statement::TypedAssignment(variables, ty, expressions) =>
                    fields.push(GenericField::try_from((variables, ty, expressions))),
                Statement::Compound(compound) => match compound.deref() {
                    CompoundStatement::Funcdef(func_def) =>
                        functions.push(GenericFunction::try_from(func_def)),
                    _ => {}
                },
                _ => {}
            }
        }

        let (fields, field_errs): (Vec<_>, Vec<_>) = fields.into_iter().partition(Result::is_ok);
        let (functions, fun_errs): (Vec<_>, Vec<_>) =
            functions.into_iter().partition(Result::is_ok);
        let (parents, parent_errs): (Vec<_>, Vec<_>) = parents.into_iter().partition(Result::is_ok);

        if !field_errs.is_empty() || !fun_errs.is_empty() || !parent_errs.is_empty() {
            let mut errs = vec![];
            errs.append(&mut field_errs.into_iter().map(Result::unwrap_err).collect());
            errs.append(&mut fun_errs.into_iter().map(Result::unwrap_err).collect());
            errs.append(&mut parent_errs.into_iter().map(Result::unwrap_err).collect());
            Err(errs.into_iter().flatten().collect())
        } else {
            Ok(GenericType {
                name,
                pos: Position::default(),
                concrete: false,
                args: vec![],
                generics: vec![],
                fields: fields.into_iter().map(Result::unwrap).collect(),
                functions: functions.into_iter().map(Result::unwrap).collect(),
                parents: parents.into_iter().map(Result::unwrap).collect()
            })
        }
    }
}
