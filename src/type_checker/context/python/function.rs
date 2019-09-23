use crate::common::position::Position;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use python_parser::ast::Funcdef;

pub const INIT: &'static str = "__init__";

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

impl From<&Funcdef> for GenericFunction {
    fn from(func_def: &Funcdef) -> GenericFunction {
        GenericFunction {
            name:      func_def.name.clone(),
            pure:      false,
            private:   false,
            pos:       Position::default(),
            generics:  vec![],
            arguments: func_def
                .parameters
                .positional_args
                .iter()
                .map(|(name, ty, expr)| GenericFunctionArg::from((name, ty, expr)))
                .collect(),
            raises:    vec![],
            ret_ty:    match &func_def.return_type {
                Some(ret_ty) => Some(GenericTypeName::from(ret_ty)),
                None => None
            }
        }
    }
}
