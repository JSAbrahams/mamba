use std::collections::HashMap;

use crate::common::position::Position;
use crate::type_checker::context::concrete::function_arg::FunctionArg;
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::type_name::GenericActualTypeName;
use crate::type_checker::type_result::TypeErr;
use std::convert::TryFrom;

pub const INIT: &'static str = "init";

pub const ADD: &'static str = "+";
pub const DIV: &'static str = "/";
pub const EQ: &'static str = "=";
pub const FDIV: &'static str = "//";
pub const GE: &'static str = ">";
pub const GEQ: &'static str = ">=";
pub const LE: &'static str = "<";
pub const LEQ: &'static str = "<=";
pub const MOD: &'static str = "mod";
pub const MUL: &'static str = "*";
pub const NEQ: &'static str = "/=";
pub const POW: &'static str = "^";
pub const SUB: &'static str = "-";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Function {
    pub is_py_type: bool,
    pub name:       String,
    pub pure:       bool,
    pub arguments:  Vec<FunctionArg>,
    pub raises:     Vec<TypeName>,
    pub ret_ty:     Option<TypeName>
}

impl Function {
    pub fn try_from(
        generic_fun: &GenericFunction,
        generics: &HashMap<String, GenericActualTypeName>,
        pos: &Position
    ) -> Result<Self, TypeErr> {
        Ok(Function {
            is_py_type: generic_fun.is_py_type,
            name:       generic_fun.name.clone(),
            pure:       generic_fun.pure,
            arguments:  generic_fun
                .arguments
                .iter()
                .map(|arg| FunctionArg::try_from(arg, generics, pos))
                .collect::<Result<_, _>>()?,
            raises:     generic_fun
                .raises
                .iter()
                .map(|raise| TypeName::try_from((raise, generics, pos)))
                .collect::<Result<_, _>>()?,
            ret_ty:     match &generic_fun.ty()? {
                Some(ty) => Some(TypeName::try_from((ty, generics, pos))?),
                None => None
            }
        })
    }
}
