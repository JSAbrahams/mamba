use std::ops::Deref;

use crate::common::position::Position;
use crate::type_checker::context::field::python::GenericFields;
use crate::type_checker::context::function;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::parent::generic::GenericParent;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::context::type_name::generic::actual::GenericActualTypeName;
use python_parser::ast::{Classdef, CompoundStatement, Name, Statement};

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
                Statement::TypedAssignment(variables, _, expressions) =>
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
            .find(|f| f.name == GenericActualTypeName::new(function::concrete::INIT))
            .map_or(vec![], |f| f.arguments.clone());

        GenericType {
            is_py_type: true,
            name: GenericActualTypeName::new(primitive_to_concrete(&class_def.name).as_str()),
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
