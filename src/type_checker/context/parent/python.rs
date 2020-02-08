use python_parser::ast::{Argument, Expression};

use crate::common::position::Position;
use crate::type_checker::context::parameter::python::GenericParameters;
use crate::type_checker::context::parent::generic::GenericParent;
use crate::type_checker::context::ty::python::python_to_concrete;
use std::ops::Deref;

impl From<&Argument> for GenericParent {
    fn from(argument: &Argument) -> GenericParent {
        let (name, generics) = match argument {
            Argument::Positional(expr) => match expr {
                Expression::Name(name) => (python_to_concrete(name), vec![]),
                Expression::Subscript(name, generics) => {
                    let name = if let Expression::Name(name) = name.deref() {
                        name.clone()
                    } else {
                        String::new()
                    };
                    let generics = GenericParameters::from(generics).parameters;
                    (python_to_concrete(&name), generics)
                }
                _ => (String::new(), vec![])
            },
            _ => (String::new(), vec![])
        };

        GenericParent { is_py_type: true, name, pos: Position::default(), generics, args: vec![] }
    }
}
