use std::convert::TryFrom;

use python_parser::ast::Decorator;

use crate::common::position::Position;
use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::parent::GenericParent;
use crate::type_checker::type_result::TypeErr;

impl TryFrom<&Decorator> for GenericParent {
    type Error = Vec<TypeErr>;

    fn try_from(decorator: &Decorator) -> Result<Self, Self::Error> {
        Ok(GenericParent {
            name:     decorator.name.get(0).cloned().unwrap_or_else(|| String::from("")),
            pos:      Position::default(),
            generics: vec![],
            args:     decorator.args.clone().map_or(Ok(vec![]), |args| {
                args.iter().map(GenericFunctionArg::try_from).collect()
            })?
        })
    }
}
