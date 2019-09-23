use python_parser::ast::{Argument, Expression};

use crate::common::position::Position;
use crate::type_checker::context::generic::parent::GenericParent;

impl From<&Argument> for GenericParent {
    fn from(argument: &Argument) -> GenericParent {
        let name = match argument {
            Argument::Positional(expr) => match expr {
                Expression::Name(name) => name.clone(),
                _ => String::new()
            },
            _ => String::new()
        };

        GenericParent { name, pos: Position::default(), generics: vec![], args: vec![] }
    }
}
