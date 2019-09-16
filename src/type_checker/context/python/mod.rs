use std::convert::TryFrom;
use std::ops::Deref;

use python_parser::ast::{CompoundStatement, Statement};

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

impl TryFrom<&Statement> for GenericType {
    type Error = Vec<TypeErr>;

    fn try_from(statement: &Statement) -> TypeResult<GenericType> {
        let mut name = String::new();
        let mut parents: Vec<Result<_, Vec<TypeErr>>> = vec![];
        let mut functions: Vec<Result<_, Vec<TypeErr>>> = vec![];
        let mut fields: Vec<Result<_, Vec<TypeErr>>> = vec![];

        match statement {
            Statement::Compound(compound_stmt) => match compound_stmt.deref() {
                CompoundStatement::Classdef(class_def) => {
                    name = class_def.name.clone();

                    for decorator in &class_def.decorators {
                        parents.push(GenericParent::try_from(decorator))
                    }

                    for statement in &class_def.code {
                        match statement {
                            Statement::Assignment(variables, expressions) =>
                                for (var, expr) in variables.iter().zip(expressions) {
                                    fields.push(GenericField::try_from(&(var, expr)))
                                },
                            Statement::TypedAssignment(variables, ty, expressions) =>
                                for (var, expr) in variables.iter().zip(expressions) {
                                    fields.push(GenericField::try_from(&(var, ty, expr)))
                                },
                            Statement::Compound(compound) => match compound.deref() {
                                CompoundStatement::Funcdef(func_def) =>
                                    functions.push(GenericFunction::try_from(func_def)),
                                _ => unimplemented!()
                            },
                            _ => unimplemented!()
                        }
                    }
                }
                _ => unimplemented!()
            },
            _ => unimplemented!()
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
