use crate::common::position::Position;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use python_parser::ast::Funcdef;
use std::convert::TryFrom;

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

impl TryFrom<&Funcdef> for GenericFunction {
    type Error = Vec<TypeErr>;

    fn try_from(func_def: &Funcdef) -> Result<Self, Self::Error> {
        Ok(GenericFunction {
            name:      func_def.name.clone(),
            pure:      false,
            private:   false,
            pos:       Position::default(),
            generics:  vec![],
            arguments: unimplemented!(),
            raises:    vec![],
            ret_ty:    match &func_def.return_type {
                Some(ret_ty) => Some(GenericTypeName::try_from(ret_ty)?),
                None => None
            }
        })
    }
}
