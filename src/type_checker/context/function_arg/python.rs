use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use python_parser::ast::Expression;

impl From<(&String, &Option<Expression>, &Option<Expression>)> for GenericFunctionArg {
    fn from(
        (name, ty, _): (&String, &Option<Expression>, &Option<Expression>)
    ) -> GenericFunctionArg {
        GenericFunctionArg {
            is_py_type: true,
            name:       name.clone(),
            pos:        Default::default(),
            vararg:     false,
            mutable:    false,
            ty:         None
        }
    }
}
