use crate::common::position::Position;
use crate::type_checker::context::type_name::generic::actual::GenericActualTypeName;
use crate::type_checker::context::type_name::generic::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;

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

impl TryFrom<(&GenericActualTypeName, &HashMap<String, GenericTypeName>, &Position)>
    for ActualTypeName
{
    type Error = TypeErr;

    fn try_from(
        (gen_type_name, generics, pos): (
            &GenericActualTypeName,
            &HashMap<String, GenericTypeName>,
            &Position
        )
    ) -> Result<Self, Self::Error> {
        let typename_from = |g| ActualTypeName::try_from(g, generics, pos);
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
                ret_ty: Box::from(ActualTypeName::try_from(ret_ty, generics, pos)?)
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
}

fn substitute(
    this: &GenericActualTypeName,
    substitute: &GenericActualTypeName,
    generics: &HashMap<String, GenericTypeName>,
    pos: &Position
) -> Result<ActualTypeName, TypeErr> {
    let typename_from = |g| ActualTypeName::try_from(g, generics, pos);
    match (this, substitute) {
        (
            GenericActualTypeName::Single { generics: this_generics, .. },
            GenericActualTypeName::Single { lit: that_lit, generics: that_generics }
        ) => Ok(ActualTypeName::Single {
            lit:      that_lit.clone(),
            generics: if this_generics.is_empty() && that_generics.is_empty() {
                vec![]
            } else if this_generics.is_empty() {
                that_generics.iter().map(typename_from).collect::<Result<_, _>>()?
            } else if that_generics.is_empty() {
                this_generics.iter().map(typename_from).collect::<Result<_, _>>()?
            } else {
                return Err(TypeErr::new(pos, "Unable to insert generic"));
            }
        }),
        (GenericActualTypeName::Single { generics: this_generics, .. }, substitute)
            if this_generics.is_empty() =>
            ActualTypeName::try_from(substitute, generics, pos),
        _ => Err(TypeErr::new(pos, "Unable to insert generic"))
    }
}
