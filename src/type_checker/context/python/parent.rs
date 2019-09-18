use std::convert::TryFrom;

use python_parser::ast::{Argument, Expression};

use crate::common::position::Position;
use crate::type_checker::context::generic::parent::GenericParent;
use crate::type_checker::type_result::TypeErr;

impl TryFrom<&Argument> for GenericParent {
    type Error = Vec<TypeErr>;

    fn try_from(argument: &Argument) -> Result<Self, Self::Error> {
        let name = match argument {
            Argument::Positional(expr) => match expr {
                Expression::Name(name) => name.clone(),
                _ => String::new()
            },
            _ => String::new()
        };

        Ok(GenericParent { name, pos: Position::default(), generics: vec![], args: vec![] })
    }
}
