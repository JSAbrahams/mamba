use std::ops::Deref;

use python_parser::ast::{Argument, Expression, Subscript};

use crate::check::context::clss::python::python_to_concrete;
use crate::check::context::parent::generic::GenericParent;
use crate::check::name::Name;
use crate::check::name::stringname::StringName;
use crate::common::position::Position;

impl From<&Argument> for GenericParent {
    fn from(argument: &Argument) -> GenericParent {
        let name = match argument {
            Argument::Positional(Expression::Name(name)) =>
                StringName::from(python_to_concrete(name).as_ref()),
            Argument::Positional(Expression::Subscript(expr, generics)) =>
                if let Expression::Name(name) = expr.deref() {
                    let generics: Vec<Name> = generics.iter().map(Name::from).collect();
                    StringName::new(python_to_concrete(name).as_ref(), &generics)
                } else {
                    StringName::empty()
                },
            _ => StringName::empty()
        };

        GenericParent { is_py_type: true, name, pos: Position::default(), args: vec![] }
    }
}

impl From<&Subscript> for Name {
    fn from(sub: &Subscript) -> Self {
        match sub {
            Subscript::Simple(expr) => Name::from(expr),
            _ => Name::empty()
        }
    }
}
