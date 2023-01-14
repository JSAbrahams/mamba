use std::ops::Deref;

use python_parser::ast::{Argument, Expression, Subscript};

use crate::check::context::clss::python::python_to_concrete;
use crate::check::context::parent::generic::GenericParent;
use crate::check::name::{Empty, Name};
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::common::position::Position;

impl From<&Argument> for GenericParent {
    fn from(argument: &Argument) -> GenericParent {
        let name = match argument {
            Argument::Positional(Expression::Name(name)) =>
                StringName::from(python_to_concrete(name).as_str()),
            Argument::Positional(Expression::Subscript(expr, generics)) =>
                if let Expression::Name(name) = expr.deref() {
                    let generics: Vec<Name> = generics.iter().map(Name::from).collect();
                    StringName::new(python_to_concrete(name).as_ref(), &generics)
                } else {
                    StringName::empty()
                },
            _ => StringName::empty()
        };

        let name = TrueName::from(&name);
        GenericParent { is_py_type: true, name, pos: Position::invisible() }
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
