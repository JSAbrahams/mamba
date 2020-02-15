use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::check::context::arg::FunctionArg;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::name::{Name, NameUnion};
use crate::check::context::{arg, function};
use crate::check::result::TypeErr;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;

pub const INIT: &str = "init";

pub const ADD: &str = "+";
pub const DIV: &str = "/";
pub const EQ: &str = "=";
pub const FDIV: &str = "//";
pub const GE: &str = ">";
pub const GEQ: &str = ">=";
pub const LE: &str = "<";
pub const LEQ: &str = "<=";
pub const MOD: &str = "mod";
pub const MUL: &str = "*";
pub const NEQ: &str = "/=";
pub const POW: &str = "^";
pub const SUB: &str = "-";
pub const SQRT: &str = "sqrt";

pub const STR: &str = function::python::STR;
pub const TRUTHY: &str = function::python::TRUTHY;
pub const NEXT: &str = function::python::NEXT;
pub const ITER: &str = function::python::ITER;

pub mod generic;
pub mod python;

/// A Function, which may either be top-level, or optionally within a class.
///
/// May return any Name within ret_ty.
/// May raise any Name within raises.
#[derive(Debug, Clone, Eq)]
pub struct Function {
    pub is_py_type:   bool,
    pub name:         Name,
    pub self_mutable: Option<bool>,
    pub private:      bool,
    pub pure:         bool,
    pub arguments:    Vec<FunctionArg>,
    pub raises:       NameUnion,
    pub in_class:     Option<Name>,
    pub ret_ty:       NameUnion
}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.arguments.hash(state);
        self.ret_ty.hash(state)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments == other.arguments && self.ret_ty == other.ret_ty
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{: >8} : ({}){}{}",
            self.name,
            comma_delimited(&self.arguments),
            if let Some(ret_ty) = &self.ret_ty { format!(" -> {}", ret_ty) } else { String::new() },
            if self.raises.is_empty() {
                String::from("")
            } else {
                format!(" raises [{}]", comma_delimited(&self.raises))
            }
        )
    }
}

impl TryFrom<(&GenericFunction, &HashMap<String, Name>, &Position)> for Function {
    type Error = Vec<TypeErr>;

    fn try_from(
        (fun, generics, pos): (&GenericFunction, &HashMap<String, Name>, &Position)
    ) -> Result<Self, Self::Error> {
        let arguments: Vec<FunctionArg> = fun
            .arguments
            .iter()
            .map(|arg| FunctionArg::try_from((arg, generics, pos)))
            .collect::<Result<_, _>>()?;

        Ok(Function {
            is_py_type: fun.is_py_type,
            name: fun.name.substitute(generics, pos)?,
            self_mutable: {
                let function_arg = arguments.iter().find_map(|a| {
                    if a.name == arg::SELF {
                        Some(a.clone())
                    } else {
                        None
                    }
                });
                function_arg.map(|a| a.mutable)
            },
            pure: fun.pure,
            private: fun.private,
            arguments,
            raises: fun
                .raises
                .iter()
                .map(|raise| raise.substitute(generics))
                .collect::<Result<_, _>>()?,
            in_class: match &fun.in_class {
                Some(in_class) => Some(in_class.substitute(generics)?),
                None => None
            },
            ret_ty: match &fun.ret_ty {
                Some(ty) => Some(ty.substitute(generics)?),
                None => None
            }
        })
    }
}
