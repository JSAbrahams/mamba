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
    pub const ADD: &'static str = GenericFunction::ADD;
    pub const DIV: &'static str = GenericFunction::DIV;
    pub const EQ: &'static str = GenericFunction::EQ;
    pub const FDIV: &'static str = GenericFunction::FDIV;
    pub const GE: &'static str = GenericFunction::GE;
    pub const GEQ: &'static str = GenericFunction::GEQ;
    pub const LE: &'static str = GenericFunction::LE;
    pub const LEQ: &'static str = GenericFunction::LEQ;
    pub const MOD: &'static str = GenericFunction::MOD;
    pub const MUL: &'static str = GenericFunction::MUL;
    pub const POW: &'static str = GenericFunction::POW;
    pub const SUB: &'static str = GenericFunction::SUB;

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
