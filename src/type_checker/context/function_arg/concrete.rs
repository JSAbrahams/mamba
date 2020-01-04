use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use itertools::{EitherOrBoth, Itertools};

// TODO make ty private again
// TODO create second pass where we assign types to function arguments using
// type inference

pub const SELF: &str = "self";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FunctionArg {
    pub is_py_type:  bool,
    pub name:        String,
    pub has_default: bool,
    pub vararg:      bool,
    pub mutable:     bool,
    pub ty:          Option<TypeName>
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

pub fn args_compatible(fun_args: &[FunctionArg], args: &[TypeName]) -> bool {
    for pair in fun_args.iter().zip_longest(args.iter()) {
        match pair {
            EitherOrBoth::Both(fun_arg, arg) =>
                if fun_arg.ty != Some(arg.clone()) {
                    return false;
                },
            EitherOrBoth::Left(fun_arg) =>
                if !fun_arg.has_default {
                    return false;
                },
            EitherOrBoth::Right(_) => return false
        }
    }
    true
}

impl FunctionArg {
    pub fn ty(&self) -> TypeResult<Option<TypeName>> {
        if self.is_py_type {
            Ok(self.ty.clone())
        } else {
            Ok(Some(
                self.ty
                    .clone()
                    .ok_or_else(|| vec![TypeErr::new_no_pos("Function argument type not given")])?
            ))
        }
    }
}

impl TryFrom<(&GenericFunctionArg, &HashMap<String, TypeName>, &Position)> for FunctionArg {
    type Error = Vec<TypeErr>;

    fn try_from(
        (fun_arg, generics, pos): (&GenericFunctionArg, &HashMap<String, TypeName>, &Position)
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
