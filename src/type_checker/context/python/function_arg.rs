use std::convert::TryFrom;

use python_parser::ast::Expression;

use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;

impl TryFrom<(&String, &Option<Expression>, &Option<Expression>)> for GenericFunctionArg {
    type Error = TypeErr;

    fn try_from(
        (name, ty, expr): (&String, &Option<Expression>, &Option<Expression>)
    ) -> Result<Self, Self::Error> {
        Ok(GenericFunctionArg {
            name:    name.clone(),
            pos:     Default::default(),
            vararg:  false,
            mutable: false,
            ty:      match ty {
                Some(ty) => Some(GenericTypeName::try_from(ty)?),
                None => None
            }
        })
    }
}
