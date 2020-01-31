use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use python_parser::ast::{Classdef, CompoundStatement, Statement};

use crate::common::position::Position;
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::context::field::generic::GenericFields;
use crate::type_checker::context::function;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::parameter::python::GenericParameters;
use crate::type_checker::context::parent::generic::GenericParent;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::ty::generic::GenericType;
use crate::type_checker::ty_name::actual::ActualTypeName;
use crate::type_checker::ty_name::TypeName;

pub const INT_PRIMITIVE: &str = "int";
pub const FLOAT_PRIMITIVE: &str = "float";
pub const STRING_PRIMITIVE: &str = "str";
pub const BOOL_PRIMITIVE: &str = "bool";
pub const ENUM_PRIMITIVE: &str = "enum";
pub const COMPLEX_PRIMITIVE: &str = "complex";

pub const RANGE: &str = "range";
pub const SET: &str = "set";
pub const LIST: &str = "list";

pub const NONE: &str = "None";
pub const EXCEPTION: &str = "Exception";

// TODO handle Python generics
impl TryFrom<&Classdef> for GenericType {
    type Error = Vec<TypeErr>;

    fn try_from(class_def: &Classdef) -> TypeResult<GenericType> {
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
                Statement::Compound(compound) =>
                    if let CompoundStatement::Funcdef(func_def) = compound.deref() {
                        functions.insert(GenericFunction::from(func_def));
                    },
                _ => {}
            }
        }

        let name =
            ActualTypeName::new(python_to_concrete(&class_def.name).as_str(), &generic_names);
        let class = TypeName::from(&name);
        let functions: Vec<GenericFunction> = functions
            .into_iter()
            .map(|f| f.in_class(Some(&class), false, &Position::default()))
            .collect::<Result<_, _>>()?;
        let args = functions
            .iter()
            .find(|f| f.name == ActualTypeName::new(function::concrete::INIT, &[]))
            .map_or(vec![], |f| f.arguments.clone());

        Ok(GenericType {
            is_py_type: true,
            name: name.clone(),
            pos: Position::default(),
            concrete: false,
            args,
            generics,
            fields,
            functions: functions
                .into_iter()
                .map(|f| f.in_class(Some(&TypeName::from(&name)), false, &Position::default()))
                .filter_map(Result::ok)
                .collect(),
            parents: class_def.arguments.iter().map(GenericParent::from).collect()
        })
    }
}

pub fn python_to_concrete(name: &str) -> String {
    match name {
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
        EXCEPTION => String::from(concrete::EXCEPTION),
        other => String::from(other)
    }
}
