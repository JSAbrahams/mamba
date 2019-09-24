use crate::common::position::Position;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::parent::GenericParent;
use crate::type_checker::context::generic::ty::GenericType;
use crate::type_checker::context::python::field::GenericFields;
use crate::type_checker::context::python::function::INIT;
use python_parser::ast::{Classdef, CompoundStatement, Statement};
use std::ops::Deref;

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
