use ruff_python_ast::ParameterWithDefault;

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::name::Name;
use crate::common::position::Position;

pub const SELF: &str = "self";

impl From<&ParameterWithDefault> for GenericFunctionArg {
    fn from(parameter: &ParameterWithDefault) -> GenericFunctionArg {
        GenericFunctionArg {
            is_py_type: true,
            name: parameter.parameter.name.as_str().to_string(),
            has_default: parameter.default.is_some(),
            pos: Position::invisible(),
            vararg: false,
            mutable: true,
            ty: parameter.parameter.annotation.as_deref().map(Name::from),
        }
    }
}
