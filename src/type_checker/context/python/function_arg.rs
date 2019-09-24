use python_parser::ast::Expression;

use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::type_name::GenericTypeName;

impl From<(&String, &Option<Expression>, &Option<Expression>)> for GenericFunctionArg {
    fn from(
        (name, ty, expr): (&String, &Option<Expression>, &Option<Expression>)
    ) -> GenericFunctionArg {
        GenericFunctionArg {
            is_py_type: true,
            name:       name.clone(),
            pos:        Default::default(),
            vararg:     false,
            mutable:    false,
            ty:         match ty {
                Some(ty) => Some(GenericTypeName::from(ty)),
                None => None
            }
        }
    }
}
