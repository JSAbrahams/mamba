use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use python_parser::ast::{Classdef, CompoundStatement, Statement};

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::GenericFields;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::parameter::python::GenericParameters;
use crate::check::context::parent::generic::GenericParent;
use crate::check::context::{clss, function};
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::name::TypeName;
use crate::common::position::Position;

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
impl TryFrom<&Classdef> for GenericClass {
    type Error = Vec<TypeErr>;

    fn try_from(class_def: &Classdef) -> TypeResult<GenericClass> {
        let mut functions = HashSet::new();
        let mut fields = HashSet::new();
        let generics = GenericParameters::from(&class_def.arguments).parameters;
        let generic_names: Vec<TypeName> =
            generics.iter().map(|g| TypeName::from(python_to_concrete(&g.name).as_str())).collect();

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

        let class = TypeName::new(python_to_concrete(&class_def.name).as_str(), &generic_names);
        let functions: Vec<GenericFunction> = functions
            .into_iter()
            .map(|f| f.in_class(Some(&class), false, &Position::default()))
            .collect::<Result<_, _>>()?;
        let args = functions
            .iter()
            .find(|f| f.name == TypeName::new(function::INIT, &[]))
            .map_or(vec![], |f| f.arguments.clone());

        Ok(GenericClass {
            is_py_type: true,
            name: class.clone(),
            pos: Position::default(),
            concrete: false,
            args,
            generics,
            fields,
            functions: functions
                .into_iter()
                .map(|f| f.in_class(Some(&class), false, &Position::default()))
                .filter_map(Result::ok)
                .collect(),
            parents: class_def.arguments.iter().map(GenericParent::from).collect()
        })
    }
}

pub fn python_to_concrete(name: &str) -> String {
    match name {
        INT_PRIMITIVE => String::from(clss::INT_PRIMITIVE),
        FLOAT_PRIMITIVE => String::from(clss::FLOAT_PRIMITIVE),
        STRING_PRIMITIVE => String::from(clss::STRING_PRIMITIVE),
        BOOL_PRIMITIVE => String::from(clss::BOOL_PRIMITIVE),
        ENUM_PRIMITIVE => String::from(clss::ENUM_PRIMITIVE),
        COMPLEX_PRIMITIVE => String::from(clss::COMPLEX_PRIMITIVE),

        RANGE => String::from(clss::RANGE),
        SET => String::from(clss::SET),
        LIST => String::from(clss::LIST),

        NONE => String::from(clss::NONE),
        EXCEPTION => String::from(clss::EXCEPTION),

        other => String::from(other)
    }
}
