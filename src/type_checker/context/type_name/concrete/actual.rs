use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;

use crate::common::position::Position;
use crate::type_checker::context::type_name::generic::actual::GenericActualTypeName;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ActualTypeName {
    Single { lit: String, generics: Vec<ActualTypeName> },
    Tuple { ty_names: Vec<ActualTypeName> },
    AnonFun { args: Vec<ActualTypeName>, ret_ty: Box<ActualTypeName> }
}

impl Display for ActualTypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ActualTypeName::Single { lit, generics } if generics.is_empty() => write!(f, "{}", lit),
            ActualTypeName::Single { lit, generics } => write!(f, "{}<{:#?}>", lit, generics),
            ActualTypeName::AnonFun { args, ret_ty } => write!(f, "({:#?}) -> {}", args, ret_ty),
            ActualTypeName::Tuple { ty_names } => write!(f, "({:#?})", ty_names)
        }
    }
}

impl From<&ActualType> for ActualTypeName {
    fn from(actual_type: &ActualType) -> Self {
        match actual_type {
            ActualType::Single { ty } => ty.name.clone(),
            ActualType::Tuple { types } => ActualTypeName::Tuple {
                ty_names: types.iter().map(|ty| ActualTypeName::from(&ty.actual_ty)).collect()
            },
            ActualType::AnonFun { args, ret_ty } => ActualTypeName::AnonFun {
                args:   args.iter().map(|arg| ActualTypeName::from(&arg.actual_ty)).collect(),
                ret_ty: Box::new(ActualTypeName::from(&ret_ty.deref().actual_ty))
            }
        }
    }
}

impl TryFrom<(&GenericActualTypeName, &HashMap<String, ActualTypeName>, &Position)>
    for ActualTypeName
{
    type Error = Vec<TypeErr>;

    fn try_from(
        (gen_type_name, generics, pos): (
            &GenericActualTypeName,
            &HashMap<String, ActualTypeName>,
            &Position
        )
    ) -> Result<Self, Self::Error> {
        let typename_from = |g| ActualTypeName::try_from((g, generics, pos));
        match gen_type_name {
            GenericActualTypeName::Single { lit, generics: this_gens } =>
                if let Some(subst) = generics.get(lit) {
                    substitute(gen_type_name, subst, generics, pos)
                } else {
                    Ok(ActualTypeName::Single {
                        lit:      lit.clone(),
                        generics: this_gens.iter().map(typename_from).collect::<Result<_, _>>()?
                    })
                },
            GenericActualTypeName::Fun { args, ret_ty } => Ok(ActualTypeName::AnonFun {
                args:   args.iter().map(typename_from).collect::<Result<_, _>>()?,
                ret_ty: Box::from(ActualTypeName::try_from((ret_ty.deref(), generics, pos))?)
            }),
            GenericActualTypeName::Tuple { ty_names } => Ok(ActualTypeName::Tuple {
                ty_names: ty_names.iter().map(typename_from).collect::<Result<_, _>>()?
            })
        }
    }
}

impl ActualTypeName {
    pub fn new(lit: &str, generics: &[ActualTypeName]) -> ActualTypeName {
        ActualTypeName::Single { lit: String::from(lit), generics: Vec::from(generics) }
    }

    pub fn name(&self, pos: &Position) -> TypeResult<String> {
        match self {
            ActualTypeName::Single { lit, .. } => Ok(lit.clone()),
            _ => Err(vec![TypeErr::new(pos, "Type does not have name")])
        }
    }
}

fn substitute(
    this: &GenericActualTypeName,
    substitute: &ActualTypeName,
    generics: &HashMap<String, ActualTypeName>,
    pos: &Position
) -> TypeResult<ActualTypeName> {
    // TODO create substitution rules for function and tuples
    match (this, substitute) {
        (
            GenericActualTypeName::Single { generics: this_generics, .. },
            ActualTypeName::Single { lit: that_lit, generics: that_generics }
        ) => Ok(ActualTypeName::Single {
            lit:      that_lit.clone(),
            generics: if this_generics.len() == that_generics.len() {
                that_generics.clone()
            } else {
                return Err(vec![TypeErr::new(pos, "Unable to insert generic")]);
            }
        }),
        _ => Err(vec![TypeErr::new(pos, "Unable to insert generic")])
    }
}
