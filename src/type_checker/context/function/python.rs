use crate::common::position::Position;
use crate::type_checker::context::function::concrete;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use python_parser::ast::{Funcdef, Name};

pub const INIT: &'static str = "__init__";

pub const ADD: &'static str = "__add__";
pub const DIV: &'static str = "__div__";
pub const EQ: &'static str = "__eq__";
pub const FDIV: &'static str = "__floordiv__";
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
            name:       ActualTypeName::new(&convert_name(&func_def.name), &vec![]),
            pure:       false,
            private:    false,
            pos:        Position::default(),
            arguments:  func_def
                .parameters
                .positional_args
                .iter()
                .map(|(name, ty, expr)| GenericFunctionArg::from((name, ty, expr)))
                .collect(),
            raises:     vec![],
            ret_ty:     match &func_def.return_type {
                Some(ret_ty) => Some(TypeName::from(ret_ty)),
                None => None
            }
        }
    }
}

fn convert_name(name: &Name) -> String {
    match name.as_str() {
        INIT => String::from(concrete::INIT),

        ADD => String::from(concrete::ADD),
        DIV => String::from(concrete::DIV),
        EQ => String::from(concrete::EQ),
        FDIV => String::from(concrete::FDIV),
        GE => String::from(concrete::GE),
        GEQ => String::from(concrete::GEQ),
        LE => String::from(concrete::LE),
        LEQ => String::from(concrete::LEQ),
        MOD => String::from(concrete::MOD),
        MUL => String::from(concrete::MUL),
        NEQ => String::from(concrete::NEQ),
        POW => String::from(concrete::POW),
        SUB => String::from(concrete::SUB),

        other => String::from(other)
    }
}
