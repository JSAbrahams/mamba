use std::ops::Deref;

use python_parser::ast::{Classdef, CompoundStatement, Name, Statement};

use crate::common::position::Position;
use crate::type_checker::context::concrete;
use crate::type_checker::context::concrete::function::INIT;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::parent::GenericParent;
use crate::type_checker::context::generic::ty::GenericType;
use crate::type_checker::context::python::field::GenericFields;

pub const INT_PRIMITIVE: &'static str = "int";
pub const FLOAT_PRIMITIVE: &'static str = "float";
pub const STRING_PRIMITIVE: &'static str = "str";
pub const BOOL_PRIMITIVE: &'static str = "bool";
pub const ENUM_PRIMITIVE: &'static str = "enum";

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
            name: primitive_to_concrete(&class_def.name),
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

fn primitive_to_concrete(name: &Name) -> String {
    match name.as_str() {
        INT_PRIMITIVE => String::from(concrete::INT_PRIMITIVE),
        FLOAT_PRIMITIVE => String::from(concrete::FLOAT_PRIMITIVE),
        STRING_PRIMITIVE => String::from(concrete::STRING_PRIMITIVE),
        BOOL_PRIMITIVE => String::from(concrete::BOOL_PRIMITIVE),
        ENUM_PRIMITIVE => String::from(concrete::ENUM_PRIMITIVE),

        other => String::from(other)
    }
}
