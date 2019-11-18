use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::function_arg;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::util::comma_delimited;
use std::hash::{Hash, Hasher};

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
pub const SQRT: &'static str = "sqrt";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Function {
    pub is_py_type:   bool,
    pub name:         ActualTypeName,
    pub self_mutable: Option<bool>,
    pub pure:         bool,
    pub arguments:    Vec<FunctionArg>,
    pub raises:       HashSet<ActualTypeName>,
    ret_ty:           Option<TypeName>
}

impl Hash for Function {
    /// Hash Function, which ignores whether function is Python type or not.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.self_mutable.hash(state);
        self.pure.hash(state);
        self.arguments.hash(state);
        self.raises.iter().for_each(|a| a.hash(state));
        self.ret_ty.hash(state);
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{: >5} : ({}){}{}",
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

impl Function {
    pub fn ty(&self) -> Option<TypeName> { self.ret_ty.clone() }
}

impl TryFrom<(&GenericFunction, &HashMap<String, TypeName>, &Position)> for Function {
    type Error = Vec<TypeErr>;

    fn try_from(
        (fun, generics, pos): (&GenericFunction, &HashMap<String, TypeName>, &Position)
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
                    if a.name == function_arg::concrete::SELF {
                        Some(a.clone())
                    } else {
                        None
                    }
                });

                function_arg.map(|a| a.mutable)
            },
            pure: fun.pure,
            arguments,
            raises: fun
                .raises
                .iter()
                .map(|raise| raise.substitute(generics, pos))
                .collect::<Result<_, _>>()?,
            ret_ty: match &fun.ret_ty {
                Some(ty) => Some(ty.substitute(generics, pos)?),
                None => None
            }
        })
    }
}
