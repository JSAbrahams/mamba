use std::fmt;
use std::fmt::{Display, Formatter};

use crate::check::context::arg::FunctionArg;
use crate::check::context::field::Field;
use crate::check::context::function::Function;
use crate::check::context::Context;
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::name::TypeName;
use crate::check::ty::nullable::NullableType;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub mod name;

pub mod actual;
pub mod nullable;

#[derive(Clone, Eq, Debug)]
pub enum Type {
    Single { ty: NullableType },
    Union { union: HashSet<NullableType> }
}

impl Hash for Type {
    /// Hash ExpressionType
    ///
    /// Due to ExpressionTypes containing HashSets, the runtime is O(n) instead
    /// of O(1).
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            Type::Single { ty } => ty.hash(state),
            Type::Union { union } => union.iter().for_each(|ty| ty.hash(state))
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Single { ty }, Type::Single { ty: other }) => ty == other,
            (Type::Union { union }, Type::Union { union: other }) => union == other,
            _ => false
        }
    }
}

impl From<&NullableType> for Type {
    fn from(nullable_type: &NullableType) -> Self { Type::Single { ty: nullable_type.clone() } }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Type::Single { ty } => write!(f, "{}", ty),
            Type::Union { union } => write!(f, "{{{}}}", comma_delimited(union))
        }
    }
}

impl Type {
    pub fn has_parent(
        &self,
        type_name: &TypeName,
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<bool> {
        // As an extra precaution
        self.has_parent_checked(type_name, &HashSet::new(), ctx, pos)
    }

    pub fn has_parent_checked(
        &self,
        type_name: &TypeName,
        checked: &HashSet<TypeName>,
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<bool> {
        if checked.contains(type_name) {
            // Should be checked during a pass of context
            let msg = format!("Circular dependency detected: {} is a parent of itself", type_name);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }

        let mut checked = checked.clone();
        checked.insert(type_name.clone());

        match &self {
            Type::Single { ty } => ty.actual_ty().has_parent(type_name, &checked, ctx, pos),
            Type::Union { union } => {
                let bools: Vec<bool> = union
                    .iter()
                    .map(|t| t.actual_ty().has_parent(type_name, &checked, ctx, pos))
                    .collect::<Result<_, _>>()?;
                Ok(bools.iter().all(|b| *b))
            }
        }
    }

    pub fn fields(&self, pos: &Position) -> TypeResult<Vec<HashSet<Field>>> {
        match &self {
            Type::Single { ty } => {
                let mut all_fields = Vec::new();
                all_fields.push(ty.actual_ty().fields(pos)?);
                Ok(all_fields)
            }
            Type::Union { union } => union.iter().map(|ty| ty.actual_ty().fields(pos)).collect()
        }
    }

    pub fn anon_fun_params(
        &self,
        pos: &Position
    ) -> TypeResult<HashSet<(Vec<TypeName>, TypeName)>> {
        match &self {
            Type::Single { ty } => {
                let mut set = HashSet::new();
                set.insert(ty.actual_ty().anon_fun_params(pos)?);
                Ok(set)
            }
            Type::Union { union } =>
                union.iter().map(|ty| ty.actual_ty().anon_fun_params(pos)).collect::<Result<_, _>>(),
        }
    }

    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        match &self {
            Type::Single { ty: mut_ty } => mut_ty.actual_ty().field(field, pos),
            Type::Union { union } => {
                let union: Vec<Field> = union
                    .iter()
                    .map(|e_ty| e_ty.actual_ty().field(field, pos))
                    .collect::<Result<_, _>>()?;
                let first = union.get(0);
                let msg = format!("Unknown field, {} does not have field {}", self, field);
                if union.iter().all(|e_ty| Some(e_ty) == first) {
                    Ok(first.cloned().ok_or_else(|| vec![TypeErr::new(pos, &msg)])?)
                } else {
                    Err(vec![TypeErr::new(pos, &msg)])
                }
            }
        }
    }

    pub fn function(&self, name: &TypeName, pos: &Position) -> TypeResult<HashSet<Function>> {
        match &self {
            Type::Single { ty } => ty.actual_ty().function(name, pos),
            Type::Union { union } => {
                let funs: Vec<Result<HashSet<_>, _>> =
                    union.into_iter().map(|t| t.actual_ty().function(name, pos)).collect();
                let res: Vec<HashSet<Function>> = funs.into_iter().collect::<Result<_, _>>()?;
                Ok(res.into_iter().flatten().collect())
            }
        }
    }

    pub fn constructor_args(&self, pos: &Position) -> TypeResult<HashSet<Vec<FunctionArg>>> {
        match &self {
            Type::Single { ty } => {
                let mut possible_args: HashSet<Vec<FunctionArg>> = HashSet::new();
                possible_args.insert(ty.constructor_args(pos)?);
                Ok(possible_args)
            }
            Type::Union { union } => {
                let constructor: HashSet<Vec<FunctionArg>> =
                    union.iter().map(|ty| ty.constructor_args(pos)).collect::<Result<_, _>>()?;
                Ok(constructor)
            }
        }
    }
}
