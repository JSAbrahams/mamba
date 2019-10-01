use python_parser::ast::{Funcdef, Name};

use crate::common::position::Position;
use crate::type_checker::context::concrete;
use crate::type_checker::context::generic::function::GenericFunction;
use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::context::generic::type_name::GenericActualTypeName;

pub const INIT: &'static str = "__init__";

pub const ADD: &'static str = "__add__";
pub const DIV: &'static str = "__div__";
pub const EQ: &'static str = "__eq__";
pub const FDIV: &'static str = "__fdiv__";
pub const GE: &'static str = "__ge__";
pub const GEQ: &'static str = "__geq__";
pub const LE: &'static str = "__le__";
pub const LEQ: &'static str = "__leq__";
pub const MOD: &'static str = "__mod__";
pub const MUL: &'static str = "__mul__";
pub const NEQ: &'static str = "__neq__";
pub const POW: &'static str = "__pow__";
pub const SUB: &'static str = "__sub__";

impl From<&Funcdef> for GenericFunction {
    fn from(func_def: &Funcdef) -> GenericFunction {
        GenericFunction {
            is_py_type: true,
            name:       convert_name(&func_def.name),
            pure:       false,
            private:    false,
            pos:        Position::default(),
            generics:   vec![],
            arguments:  func_def
                .parameters
                .positional_args
                .iter()
                .map(|(name, ty, expr)| GenericFunctionArg::from((name, ty, expr)))
                .collect(),
            raises:     vec![],
            ret_ty:     match &func_def.return_type {
                Some(ret_ty) => Some(GenericActualTypeName::from(ret_ty)),
                None => None
            }
        }
    }
}

fn convert_name(name: &Name) -> String {
    match name.as_str() {
        INIT => String::from(concrete::function::INIT),

        ADD => String::from(concrete::function::ADD),
        DIV => String::from(concrete::function::DIV),
        EQ => String::from(concrete::function::EQ),
        FDIV => String::from(concrete::function::FDIV),
        GE => String::from(concrete::function::GE),
        GEQ => String::from(concrete::function::GEQ),
        LE => String::from(concrete::function::LE),
        LEQ => String::from(concrete::function::LEQ),
        MOD => String::from(concrete::function::MOD),
        MUL => String::from(concrete::function::MUL),
        NEQ => String::from(concrete::function::NEQ),
        POW => String::from(concrete::function::POW),
        SUB => String::from(concrete::function::SUB),

        other => String::from(other)
    }
}
