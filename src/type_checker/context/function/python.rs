use crate::common::position::Position;
use crate::type_checker::context::function::concrete;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use crate::type_checker::ty_name::actual::ActualTypeName;
use crate::type_checker::ty_name::TypeName;
use python_parser::ast::Funcdef;

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

pub const TRUTHY: &str = "__bool__";
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
        POW => String::from(concrete::POW),
        SUB => String::from(concrete::SUB),

        TRUTHY => String::from(concrete::TRUTHY),

        other => String::from(other)
    }
}
