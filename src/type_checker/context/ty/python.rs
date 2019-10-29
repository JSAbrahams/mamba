use std::collections::HashSet;
use std::ops::Deref;

use python_parser::ast::{Classdef, CompoundStatement, Name, Statement};

use crate::common::position::Position;
use crate::type_checker::context::field::python::GenericFields;
use crate::type_checker::context::function;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::parameter::python::GenericParameters;
use crate::type_checker::context::parent::generic::GenericParent;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;

pub const INT_PRIMITIVE: &'static str = "int";
pub const FLOAT_PRIMITIVE: &'static str = "float";
pub const STRING_PRIMITIVE: &'static str = "str";
pub const BOOL_PRIMITIVE: &'static str = "bool";
pub const ENUM_PRIMITIVE: &'static str = "enum";
pub const COMPLEX_PRIMITIVE: &'static str = "complex";

pub const RANGE: &'static str = "range";
pub const SET: &'static str = "set";
pub const LIST: &'static str = "list";

pub const NONE: &'static str = "None";

// TODO handle Python generics
impl From<&Classdef> for GenericType {
    fn from(class_def: &Classdef) -> GenericType {
        let mut functions = HashSet::new();
        let mut fields = HashSet::new();
        let generics = GenericParameters::from(&class_def.arguments).parameters;
        let generic_names: Vec<TypeName> =
            generics.iter().map(|g| TypeName::from(g.name.as_str())).collect();

        for statement in &class_def.code {
            match statement {
                Statement::Assignment(variables, expressions) =>
                    GenericFields::from((variables, expressions)).fields.iter().for_each(|f| {
                        fields.insert(f.clone());
                    }),
                Statement::TypedAssignment(variables, _, expressions) =>
                    GenericFields::from((variables, expressions)).fields.iter().for_each(|f| {
                        fields.insert(f.clone());
                    }),
                Statement::Compound(compound) => match compound.deref() {
                    CompoundStatement::Funcdef(func_def) => {
                        functions.insert(GenericFunction::from(func_def));
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let args = functions
            .iter()
            .find(|f| f.name == ActualTypeName::new(function::concrete::INIT, &vec![]))
            .map_or(vec![], |f| f.arguments.clone());

        let name =
            ActualTypeName::new(python_to_concrete(&class_def.name).as_str(), &generic_names);
        GenericType {
            is_py_type: true,
            name: name.clone(),
            pos: Position::default(),
            concrete: false,
            args,
            generics,
            fields,
            functions: functions
                .into_iter()
                .map(|f| f.in_class(Some(&TypeName::from(&name)), &Position::default()))
                .filter_map(Result::ok)
                .collect(),
            parents: class_def.arguments.iter().map(GenericParent::from).collect()
        }
    }
}

pub fn python_to_concrete(name: &Name) -> String {
    match name.as_str() {
        INT_PRIMITIVE => String::from(concrete::INT_PRIMITIVE),
        FLOAT_PRIMITIVE => String::from(concrete::FLOAT_PRIMITIVE),
        STRING_PRIMITIVE => String::from(concrete::STRING_PRIMITIVE),
        BOOL_PRIMITIVE => String::from(concrete::BOOL_PRIMITIVE),
        ENUM_PRIMITIVE => String::from(concrete::ENUM_PRIMITIVE),
        COMPLEX_PRIMITIVE => String::from(concrete::COMPLEX_PRIMITIVE),

        RANGE => String::from(concrete::RANGE),
        SET => String::from(concrete::SET),
        LIST => String::from(concrete::LIST),

        NONE => String::from(concrete::NONE),
        other => String::from(other)
    }
}
