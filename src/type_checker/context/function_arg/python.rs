use python_parser::ast::Expression;

use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use crate::type_checker::context::ty::python::python_to_concrete;
use crate::type_checker::context::type_name::TypeName;

impl From<(&String, &Option<Expression>, &Option<Expression>)> for GenericFunctionArg {
    fn from(
        (name, ty, _): (&String, &Option<Expression>, &Option<Expression>)
    ) -> GenericFunctionArg {
        // TODO creat function for extracting union from expressions
        GenericFunctionArg {
            is_py_type: true,
            name:       name.clone(),
            pos:        Default::default(),
            vararg:     false,
            mutable:    false,
            ty:         ty.clone().map(|e| TypeName::from(&e))
        }
    }
}
