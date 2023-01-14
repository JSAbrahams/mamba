use python_parser::ast::Expression;

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::name::Name;
use crate::common::position::Position;

pub const SELF: &str = "self";

impl From<(&String, &Option<Expression>, &Option<Expression>)> for GenericFunctionArg {
    fn from(
        (name, ty, default): (&String, &Option<Expression>, &Option<Expression>)
    ) -> GenericFunctionArg {
        GenericFunctionArg {
            is_py_type: true,
            name: name.clone(),
            has_default: default.is_some(),
            pos: Position::invisible(),
            vararg: false,
            mutable: true,
            ty: ty.clone().map(|e| Name::from(&e)),
        }
    }
}
