use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::context::name::{Name, NameUnion};
use crate::check::result::TypeErr;
use crate::common::position::Position;

pub const SELF: &str = "self";

pub mod generic;
pub mod python;

/// A Function argument.
///
/// May have a type.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FunctionArg {
    pub is_py_type:  bool,
    pub name:        String,
    pub has_default: bool,
    pub vararg:      bool,
    pub mutable:     bool,
    pub ty:          Option<NameUnion>
}

impl Display for FunctionArg {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.name,
            if let Some(ty) = &self.ty { format!(": {}", ty) } else { String::new() },
            if self.has_default { "?" } else { "" }
        )
    }
}

impl TryFrom<(&GenericFunctionArg, &HashMap<String, Name>, &Position)> for FunctionArg {
    type Error = Vec<TypeErr>;

    fn try_from(
        (fun_arg, generics, pos): (&GenericFunctionArg, &HashMap<String, Name>, &Position)
    ) -> Result<Self, Self::Error> {
        Ok(FunctionArg {
            is_py_type:  fun_arg.is_py_type,
            name:        fun_arg.name.clone(),
            has_default: fun_arg.has_default,
            vararg:      fun_arg.vararg,
            mutable:     fun_arg.mutable,
            ty:          match &fun_arg.ty {
                Some(ty) => Some(ty.substitute(generics, pos)?),
                None => None
            }
        })
    }
}
