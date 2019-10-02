use crate::common::position::Position;
use crate::type_checker::context::function_arg::generic::GenericFunctionArg;
use crate::type_checker::context::type_name::concrete::actual::ActualTypeName;
use crate::type_checker::context::type_name::concrete::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::HashMap;
use std::convert::TryFrom;

// TODO make ty private again

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FunctionArg {
    pub is_py_type: bool,
    pub name:       String,
    pub vararg:     bool,
    pub mutable:    bool,
    pub ty:         Option<TypeName>
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

impl TryFrom<(&GenericFunctionArg, &HashMap<String, ActualTypeName>, &Position)> for FunctionArg {
    type Error = Vec<TypeErr>;

    fn try_from(
        (fun_arg, generics, pos): (
            &GenericFunctionArg,
            &HashMap<String, ActualTypeName>,
            &Position
        )
    ) -> Result<Self, Self::Error> {
        Ok(FunctionArg {
            is_py_type: fun_arg.is_py_type,
            name:       fun_arg.name.clone(),
            vararg:     fun_arg.vararg,
            mutable:    fun_arg.mutable,
            ty:         match &fun_arg.ty {
                Some(ty) => Some(TypeName::try_from((ty, generics, pos))?),
                None => None
            }
        })
    }
}
