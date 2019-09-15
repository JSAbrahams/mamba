use std::collections::HashMap;

use crate::common::position::Position;
use crate::type_checker::context::generic_function::GenericFunction;
use crate::type_checker::context::generic_type_name::GenericTypeName;
use crate::type_checker::environment::function_arg::FunctionArg;
use crate::type_checker::environment::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone)]
pub struct Function {
    pub name:      String,
    pub pure:      bool,
    pub arguments: Vec<FunctionArg>,
    pub raises:    Vec<TypeName>,
    pub ret_ty:    Option<TypeName>
}

impl Function {
    pub fn try_from(
        generic_fun: &GenericFunction,
        generics: &HashMap<String, GenericTypeName>,
        pos: &Position
    ) -> Result<Self, TypeErr> {
        Ok(Function {
            name:      generic_fun.name.clone(),
            pure:      generic_fun.pure,
            arguments: generic_fun
                .arguments
                .iter()
                .map(|arg| FunctionArg::try_from(arg, generics, pos))
                .collect::<Result<_, _>>()?,
            raises:    generic_fun
                .raises
                .iter()
                .map(|raise| TypeName::try_from(raise, generics, pos))
                .collect::<Result<_, _>>()?,
            ret_ty:    Some(TypeName::try_from(&generic_fun.ty()?, generics, pos)?)
        })
    }
}
