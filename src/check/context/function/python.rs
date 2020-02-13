use python_parser::ast::Funcdef;

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::context::function;
use crate::check::context::function::generic::GenericFunction;
use crate::check::ty::name::actual::ActualTypeName;
use crate::check::ty::name::TypeName;
use crate::common::position::Position;

pub const INIT: &str = "__init__";

pub const ADD: &str = "__add__";
pub const DIV: &str = "__div__";
pub const EQ: &str = "__eq__";
pub const FDIV: &str = "__floordiv__";
pub const GE: &str = "__ge__";
pub const GEQ: &str = "__geq__";
pub const LE: &str = "__le__";
pub const LEQ: &str = "__leq__";
pub const MOD: &str = "__mod__";
pub const MUL: &str = "__mul__";
pub const POW: &str = "__pow__";
pub const SUB: &str = "__sub__";

pub const STR: &str = "__str__";
pub const TRUTHY: &str = "__bool__";
pub const NEXT: &str = "__next__";
pub const ITER: &str = "__iter__";

impl From<&Funcdef> for GenericFunction {
    fn from(func_def: &Funcdef) -> GenericFunction {
        GenericFunction {
            is_py_type: true,
            name:       ActualTypeName::new(&convert_name(&func_def.name), &[]),
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
            in_class:   None,
            ret_ty:     match &func_def.return_type {
                Some(ret_ty) => Some(TypeName::from(ret_ty)),
                None => None
            }
        }
    }
}

fn convert_name(name: &str) -> String {
    match name {
        INIT => String::from(function::INIT),

        ADD => String::from(function::ADD),
        DIV => String::from(function::DIV),
        EQ => String::from(function::EQ),
        FDIV => String::from(function::FDIV),
        GE => String::from(function::GE),
        GEQ => String::from(function::GEQ),
        LE => String::from(function::LE),
        LEQ => String::from(function::LEQ),
        MOD => String::from(function::MOD),
        MUL => String::from(function::MUL),
        POW => String::from(function::POW),
        SUB => String::from(function::SUB),

        TRUTHY => String::from(function::TRUTHY),

        other => String::from(other)
    }
}
