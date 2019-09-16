use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use python_parser::ast::Expression;
use std::convert::TryFrom;

impl TryFrom<&Expression> for GenericTypeName {
    type Error = Vec<TypeErr>;

    fn try_from(value: &Expression) -> Result<Self, Self::Error> { unimplemented!() }
}
