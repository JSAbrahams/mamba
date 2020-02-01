use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use crate::common::position::Position;
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::ty::concrete;
use crate::type_checker::ty::nullable::NullableType;
use crate::type_checker::ty_name::actual::ActualTypeName;
use crate::type_checker::ty_name::nullable::NullableTypeName;
use crate::type_checker::ty_name::TypeName;
use crate::type_checker::util::comma_delimited;

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
    pub fn union(self, other: &ExpressionType) -> ExpressionType {
        let union: HashSet<NullableType> = match (&self, other) {
            (ExpressionType::Single { ty: mut_ty }, ExpressionType::Single { ty: other }) =>
                HashSet::from_iter(vec![mut_ty.clone(), other.clone()].into_iter()),
            (ExpressionType::Single { ty }, ExpressionType::Union { union })
            | (ExpressionType::Union { union }, ExpressionType::Single { ty }) =>
                union.union(&HashSet::from_iter(vec![ty.clone()].into_iter())).cloned().collect(),
            (ExpressionType::Union { union }, ExpressionType::Union { union: other }) =>
                union.union(other).cloned().collect(),
        };

        // TODO check if parent of type is Exception
        let union = make_nullable_if_none(&union);
        let union = remove_exception(&union);
        if union.len() == 1 {
            let ty: NullableType = Vec::from_iter(union.iter())[0].clone();
            ExpressionType::Single { ty }
        } else {
            ExpressionType::Union { union }
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

    pub fn single(&self, pos: &Position) -> TypeResult<NullableType> {
        match self {
            ExpressionType::Single { ty: mut_ty } => Ok(mut_ty.clone()),
            ExpressionType::Union { .. } => Err(vec![TypeErr::new(pos, "Cannot be union")])
        }
    }

    pub fn is_nullable(&self) -> bool {
        match self {
            ExpressionType::Single { ty: mut_ty } => mut_ty.is_nullable,
            ExpressionType::Union { union } =>
                !union.is_empty() && union.iter().all(|mut_ty| mut_ty.is_nullable),
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

    pub fn anon_fun(&self, args: &[TypeName], pos: &Position) -> TypeResult<ExpressionType> {
        match &self {
            ExpressionType::Single { ty } => ty.actual_ty().anon_fun(args, pos),
            ExpressionType::Union { union } => {
                let ret_tys: Vec<ExpressionType> = union
                    .iter()
                    .map(|ty| ty.actual_ty().anon_fun(args, pos))
                    .collect::<Result<_, _>>()?;
                let mut ret_ty = ret_tys
                    .first()
                    .ok_or_else(|| vec![TypeErr::new(pos, "Union is empty")])?
                    .clone();
                for ty in ret_tys {
                    ret_ty = ret_ty.union(&ty);
                }
                Ok(ret_ty)
            }
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

    pub fn fun_args(
        &self,
        name: &TypeName,
        pos: &Position
    ) -> TypeResult<HashSet<Vec<FunctionArg>>> {
        match &self {
            ExpressionType::Single { ty } => {
                let mut set = HashSet::new();
                set.insert(ty.actual_ty().fun_args(name, pos)?);
                Ok(set)
            }
            ExpressionType::Union { union } => union
                .iter()
                .map(|e_ty| e_ty.actual_ty().fun_args(name, pos))
                .collect::<Result<_, Vec<TypeErr>>>()
        }
    }

    pub fn fun_ret_ty(&self, name: &TypeName, pos: &Position) -> TypeResult<HashSet<TypeName>> {
        match &self {
            ExpressionType::Single { ty } => {
                let mut set = HashSet::new();
                if let Some(ret_ty) = ty.actual_ty().fun_ret_ty(name, pos)? {
                    set.insert(ret_ty);
                }
                Ok(set)
            }
            ExpressionType::Union { union } => Ok(union
                .iter()
                .map(|e_ty| e_ty.actual_ty().fun_ret_ty(name, pos))
                .collect::<Result<Vec<Option<TypeName>>, Vec<TypeErr>>>()?
                .into_iter()
                .filter_map(|ret_ty: Option<TypeName>| ret_ty)
                .collect())
        }
    }

    // TODO use ActualTypeName
    pub fn fun(&self, name: &str, args: &[TypeName], pos: &Position) -> TypeResult<Function> {
        match &self {
            ExpressionType::Single { ty } => ty.actual_ty().fun(name, args, pos),
            ExpressionType::Union { union } => {
                let union: Vec<Function> = union
                    .iter()
                    .map(|e_ty| e_ty.actual_ty().fun(name, args, pos))
                    .collect::<Result<_, Vec<TypeErr>>>()?;
                let first = union.get(0);

                if union.iter().all(|e_ty| Some(e_ty) == first) {
                    Ok(first.cloned().ok_or_else(|| vec![TypeErr::new(pos, "Unknown function")])?)
                } else {
                    Err(vec![TypeErr::new(pos, "Unknown field")])
                }
            }
        }
    }

    pub fn constructor(&self, args: &[TypeName], pos: &Position) -> TypeResult<ExpressionType> {
        Ok(match &self {
            ExpressionType::Single { ty } =>
                ExpressionType::Single { ty: ty.constructor(args, pos)? },
            ExpressionType::Union { union } => ExpressionType::Union {
                union: union
                    .iter()
                    .map(|ty| ty.constructor(args, pos))
                    .collect::<Result<_, _>>()?
            }
        })
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

fn make_nullable_if_none(union: &HashSet<NullableType>) -> HashSet<NullableType> {
    let any_nullable = union.iter().any(|ty| {
        ActualTypeName::from(&ty.actual_ty()) == ActualTypeName::new(concrete::NONE, &[])
    });
    let union = if any_nullable {
        union.iter().map(NullableType::as_nullable).collect()
    } else {
        union.clone()
    };

    if union.len() == 1 {
        union
    } else {
        union
            .into_iter()
            .filter(|ty| {
                NullableTypeName::from(ty).actual != ActualTypeName::new(concrete::NONE, &[])
            })
            .collect()
    }
}

fn remove_exception(union: &HashSet<NullableType>) -> HashSet<NullableType> {
    if union.len() == 1 {
        union.clone()
    } else {
        union
            .clone()
            .into_iter()
            .filter(|ty| {
                NullableTypeName::from(ty).actual != ActualTypeName::new(concrete::EXCEPTION, &[])
            })
            .collect()
    }
}