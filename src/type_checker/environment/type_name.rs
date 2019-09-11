use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

use crate::common::position::Position;
use crate::type_checker::context::generic_type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    Single { lit: String, generics: Vec<TypeName> },
    Fun { args: Vec<TypeName>, ret_ty: Box<TypeName> },
    Tuple { type_names: Vec<TypeName> }
}

impl TypeName {
    pub fn try_from(
        gen_type_name: &GenericTypeName,
        generics: &HashMap<String, GenericTypeName>,
        pos: &Position
    ) -> Result<TypeName, TypeErr> {
        match gen_type_name {
            GenericTypeName::Single { lit, generics: this_gens } => generics.get(lit).map_or(
                Ok(TypeName::Single {
                    lit:      lit.clone(),
                    generics: this_gens
                        .iter()
                        .map(|g| TypeName::try_from(g, generics, pos))
                        .collect::<Result<_, _>>()?
                }),
                |generic_type_name| match generic_type_name {
                    GenericTypeName::Single { lit, generics: other_gens } =>
                        if other_gens.len() == this_gens.len() {
                            Ok(TypeName::Single {
                                lit:      lit.clone(),
                                generics: this_gens
                                    .iter()
                                    .map(|g| TypeName::try_from(g, generics, pos))
                                    .collect::<Result<_, _>>()?
                            })
                        } else {
                            Err(TypeErr::new(
                                pos,
                                "Trying to insert generic with unequal amount of generic arguments"
                            ))
                        },
                    _ => Err(TypeErr::new(pos, "Unable to insert generic"))
                }
            ),
            GenericTypeName::Fun { args, ret_ty } => Ok(TypeName::Fun {
                args:   args
                    .iter()
                    .map(|a| TypeName::try_from(a, generics, pos))
                    .collect::<Result<_, _>>()?,
                ret_ty: Box::from(TypeName::try_from(ret_ty, generics, pos)?)
            }),
            GenericTypeName::Tuple { type_names } => Ok(TypeName::Tuple {
                type_names: type_names
                    .iter()
                    .map(|t| TypeName::try_from(t, generics, pos))
                    .collect::<Result<_, _>>()?
            })
        }
    }
}

impl Display for TypeName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeName::Single { lit, generics } if generics.is_empty() => write!(f, "{}", lit),
            TypeName::Single { lit, generics } =>
                write!(f, "{}<{}>", lit, comma_delimited(generics)?),
            TypeName::Fun { args, ret_ty } =>
                write!(f, "({}) -> {}", comma_delimited(args)?, ret_ty),
            TypeName::Tuple { type_names } => write!(f, "({})", comma_delimited(type_names)?)
        }
    }
}

fn comma_delimited(types: &[TypeName]) -> Result<String, fmt::Error> {
    let mut res = String::new();
    for ty in types {
        res.push_str(format!("{}", ty).as_str());
        res.push(',');
        res.push(' ');
    }

    if res.len() > 1 {
        res.remove(res.len() - 1);
        res.remove(res.len() - 1);
    }
    Ok(res)
}
