use python_parser::ast::{Argument, Expression};

use crate::check::context::clss::python::python_to_concrete;
use crate::check::context::name::DirectName;
use crate::check::context::parameter::python::GenericParameters;
use crate::check::context::parent::generic::GenericParent;
use crate::common::position::Position;
use std::ops::Deref;

impl From<&Argument> for GenericParent {
    fn from(argument: &Argument) -> GenericParent {
        let (name, generics) = match argument {
            Argument::Positional(expr) => match expr {
                Expression::Name(name) =>
                    (DirectName::from(python_to_concrete(name).as_ref()), vec![]),
                Expression::Subscript(name, generics) => {
                    let name = if let Expression::Name(name) = name.deref() {
                        DirectName::from(python_to_concrete(name).as_ref())
                    } else {
                        DirectName::empty()
                    };
                    let generics = GenericParameters::from(generics).parameters;
                    (name, generics)
                }
                _ => (DirectName::empty(), vec![])
            },
            _ => (DirectName::empty(), vec![])
        };

        GenericParent { is_py_type: true, name, pos: Position::default(), generics, args: vec![] }
    }
}
