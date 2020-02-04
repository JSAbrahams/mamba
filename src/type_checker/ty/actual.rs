use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use crate::common::position::Position;
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::function_arg::concrete::args_compatible;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::ty::concrete::Type;
use crate::type_checker::context::Context;
use crate::type_checker::ty::expression::ExpressionType;
use crate::type_checker::ty_name::TypeName;
use crate::type_checker::util::comma_delimited;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ActualType {
    Single { ty: Type },
    Tuple { types: Vec<ExpressionType> },
    AnonFun { args: Vec<ExpressionType>, ret_ty: Box<ExpressionType> }
}

impl Display for ActualType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ActualType::Single { ty } => write!(f, "{}", ty),
            ActualType::Tuple { types } => write!(f, "({})", comma_delimited(types)),
            ActualType::AnonFun { args, ret_ty } =>
                write!(f, "({}) -> {}", comma_delimited(args), ret_ty),
        }
    }
}

impl ActualType {
    /// Has parent if:
    /// - Self name is `type_name`
    /// - Immediate parent is `type_name`
    /// - One of parents has `type_name` as parent
    pub fn has_parent(
        &self,
        type_name: &TypeName,
        ctx: &Context,
        pos: &Position
    ) -> TypeResult<bool> {
        match &self {
            ActualType::Single { ty } => {
                let immediate_parents = ty.parents.clone();
                Ok(TypeName::from(&ty.name) == type_name.clone()
                    || immediate_parents.contains(type_name)
                    || {
                        let parent_tys: Vec<ExpressionType> = immediate_parents
                            .iter()
                            .map(|p| ctx.lookup(p, pos))
                            .collect::<Result<_, _>>()?;
                        let bools: Vec<bool> = parent_tys
                            .iter()
                            .map(|p_ty| p_ty.has_parent(type_name, ctx, pos))
                            .collect::<Result<_, _>>()?;
                        bools.iter().any(|b| *b)
                    })
            }
            _ => Err(vec![TypeErr::new(pos, &format!("{} does not have parents", &self))])
        }
    }

    pub fn fields(&self, pos: &Position) -> TypeResult<HashSet<Field>> {
        match &self {
            ActualType::Single { ty } => Ok(ty.fields.clone()),
            _ => Err(vec![TypeErr::new(pos, &format!("{} does not have fields", &self))])
        }
    }

    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        match &self {
            ActualType::Single { ty } => ty.field(field, pos),
            _ => Err(vec![TypeErr::new(pos, &format!("{} cannot have field: {}", &self, field))])
        }
    }

    pub fn anon_fun_params(&self, pos: &Position) -> TypeResult<(Vec<TypeName>, TypeName)> {
        match &self {
            ActualType::AnonFun { args, ret_ty } =>
                Ok((args.iter().map(TypeName::from).collect(), TypeName::from(ret_ty.deref()))),
            _ => Err(vec![TypeErr::new(pos, "Not an anonymous function")])
        }
    }

    pub fn anon_fun(&self, args: &[TypeName], pos: &Position) -> TypeResult<ExpressionType> {
        match &self {
            ActualType::AnonFun { args: a, ret_ty } => {
                let fun_args = a.iter().map(TypeName::from).collect::<Vec<TypeName>>();
                if fun_args == args {
                    Ok(ret_ty.deref().clone())
                } else {
                    let msg = format!(
                        "Anonymous function expected ({}), but got ({})",
                        comma_delimited(fun_args),
                        comma_delimited(args)
                    );
                    Err(vec![TypeErr::new(pos, &msg)])
                }
            }
            _ => Err(vec![TypeErr::new(pos, "Not an anonymous function")])
        }
    }

    pub fn fun_args(&self, name: &TypeName, pos: &Position) -> TypeResult<Vec<FunctionArg>> {
        match &self {
            ActualType::Single { ty } => ty.fun_args(name, pos),
            _ => Err(vec![TypeErr::new(pos, &format!("Undefined function: {}", name))])
        }
    }

    pub fn fun_ret_ty(&self, name: &TypeName, pos: &Position) -> TypeResult<Option<TypeName>> {
        match &self {
            ActualType::Single { ty } => ty.fun_ret_ty(name, pos),
            _ => Err(vec![TypeErr::new(pos, &format!("Undefined function: {}", name))])
        }
    }

    pub fn fun(&self, name: &str, args: &[TypeName], pos: &Position) -> TypeResult<Function> {
        match &self {
            ActualType::Single { ty } => ty.fun(name, args, pos),
            _ => Err(vec![TypeErr::new(pos, &format!("Undefined function: {}", name))])
        }
    }

    pub fn constructor_args(&self, pos: &Position) -> TypeResult<Vec<FunctionArg>> {
        match &self {
            ActualType::Single { ty } => Ok(ty.args.clone()),
            _ => Err(vec![TypeErr::new(pos, "Type does not have constructor arguments")])
        }
    }

    pub fn constructor(&self, args: &[TypeName], pos: &Position) -> TypeResult<ActualType> {
        match &self {
            ActualType::Single { ty } => {
                let mut new_args = vec![TypeName::from(&ty.name)];
                new_args.append(&mut args.to_vec());

                if args_compatible(&ty.args, &new_args) {
                    Ok(self.clone())
                } else {
                    Err(vec![TypeErr::new(
                        pos,
                        &format!(
                            "{} only takes arguments ({}). Was given: ({}).",
                            ty.clone(),
                            comma_delimited(&ty.args),
                            comma_delimited(new_args)
                        )
                    )])
                }
            }
            _ => Err(vec![TypeErr::new(pos, "Type does not have constructor arguments")])
        }
    }
}
