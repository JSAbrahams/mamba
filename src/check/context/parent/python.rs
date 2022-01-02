use std::ops::Deref;

use python_parser::ast::{Argument, Expression, Subscript};

use crate::check::context::clss::python::python_to_concrete;
use crate::check::context::name::{DirectName, NameUnion};
use crate::check::context::parent::generic::GenericParent;
use crate::common::position::Position;

impl From<&Argument> for GenericParent {
    fn from(argument: &Argument) -> GenericParent {
        let name = match argument {
            Argument::Positional(Expression::Name(name)) =>
                DirectName::from(python_to_concrete(name).as_ref()),
            Argument::Positional(Expression::Subscript(expr, generics)) =>
                if let Expression::Name(name) = expr.deref() {
                    let generics: Vec<NameUnion> = generics.iter().map(NameUnion::from).collect();
                    DirectName::new(python_to_concrete(name).as_ref(), &generics)
                } else {
                    DirectName::empty()
                },
            _ => DirectName::empty()
        };

        GenericParent { is_py_type: true, name, pos: Position::default(), args: vec![] }
    }
}

impl From<&Subscript> for NameUnion {
    fn from(sub: &Subscript) -> Self {
        match sub {
            Subscript::Simple(expr) => NameUnion::from(expr),
            _ => NameUnion::empty()
        }
    }
}
