use std::fmt;
use std::fmt::{Display, Formatter};

use crate::check::checker_result::{TypeErr, TypeResult};
use crate::check::context::field::concrete::Field;
use crate::check::context::function::concrete::Function;
use crate::check::context::function_arg::concrete::FunctionArg;
use crate::check::context::Context;
use crate::check::ty::concrete::nullable::NullableType;
use crate::check::ty::name::TypeName;
use crate::common::delimit::comma_delimited;
use crate::common::position::Position;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub mod actual;
pub mod nullable;

#[derive(Clone, Eq, Debug)]
pub enum ExpressionType {
    Single { ty: NullableType },
    Union { union: HashSet<NullableType> }
}

impl Hash for ExpressionType {
    /// Hash ExpressionType
    ///
    /// Due to ExpressionTypes containing HashSets, the runtime is O(n) instead
    /// of O(1).
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            ExpressionType::Single { ty } => ty.hash(state),
            ExpressionType::Union { union } => union.iter().for_each(|ty| ty.hash(state))
        }
    }
}

impl PartialEq for ExpressionType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExpressionType::Single { ty }, ExpressionType::Single { ty: other }) => ty == other,
            (ExpressionType::Union { union }, ExpressionType::Union { union: other }) =>
                union == other,
            _ => false
        }
    }
}

impl From<&NullableType> for ExpressionType {
    fn from(nullable_type: &NullableType) -> Self {
        ExpressionType::Single { ty: nullable_type.clone() }
    }
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ExpressionType::Single { ty } => write!(f, "{}", ty),
            ExpressionType::Union { union } => write!(f, "{{{}}}", comma_delimited(union))
        }
    }
}

impl ExpressionType {
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
            ExpressionType::Single { ty } =>
                ty.actual_ty().has_parent(type_name, &checked, ctx, pos),
            ExpressionType::Union { union } => {
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
            ExpressionType::Single { ty } => {
                let mut all_fields = Vec::new();
                all_fields.push(ty.actual_ty().fields(pos)?);
                Ok(all_fields)
            }
            ExpressionType::Union { union } =>
                union.iter().map(|ty| ty.actual_ty().fields(pos)).collect(),
        }
    }

    pub fn anon_fun_params(
        &self,
        pos: &Position
    ) -> TypeResult<HashSet<(Vec<TypeName>, TypeName)>> {
        match &self {
            ExpressionType::Single { ty } => {
                let mut set = HashSet::new();
                set.insert(ty.actual_ty().anon_fun_params(pos)?);
                Ok(set)
            }
            ExpressionType::Union { union } =>
                union.iter().map(|ty| ty.actual_ty().anon_fun_params(pos)).collect::<Result<_, _>>(),
        }
    }

    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        match &self {
            ExpressionType::Single { ty: mut_ty } => mut_ty.actual_ty().field(field, pos),
            ExpressionType::Union { union } => {
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
            ExpressionType::Single { ty } => ty.actual_ty().function(name, pos),
            ExpressionType::Union { union } => {
                let funs: Vec<Result<HashSet<_>, _>> =
                    union.into_iter().map(|t| t.actual_ty().function(name, pos)).collect();
                let res: Vec<HashSet<Function>> = funs.into_iter().collect::<Result<_, _>>()?;
                Ok(res.into_iter().flatten().collect())
            }
        }
    }

    pub fn constructor_args(&self, pos: &Position) -> TypeResult<HashSet<Vec<FunctionArg>>> {
        match &self {
            ExpressionType::Single { ty } => {
                let mut possible_args: HashSet<Vec<FunctionArg>> = HashSet::new();
                possible_args.insert(ty.constructor_args(pos)?);
                Ok(possible_args)
            }
            ExpressionType::Union { union } => {
                let constructor: HashSet<Vec<FunctionArg>> =
                    union.iter().map(|ty| ty.constructor_args(pos)).collect::<Result<_, _>>()?;
                Ok(constructor)
            }
        }
    }
}
