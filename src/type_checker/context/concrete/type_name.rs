use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

use crate::common::position::Position;
use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypeName {
    Single { lit: String, generics: Vec<TypeName> },
    Fun { args: Vec<TypeName>, ret_ty: Box<TypeName> },
    Tuple { ty_names: Vec<TypeName> }
}

impl TypeName {
    /// Insert generics into the [`GenericTypeName`] to produce actual types.
    ///
    /// Traverses [GenericTypeName]'s until we find a
    /// [`Single`] variant. If its literal is present in the [`HashMap`] of
    /// generics, which maps the substitute's name to a generic argument, it
    /// may be substituted. The rules of substitution are as follows:
    ///
    /// - If the current [`GenericTypeName`] has no generics, it is a trivial
    ///   substitution, and we recursively traverse the generics of the
    ///   resulting value if relevant.
    /// - If the current [`GenericTypeName`] has generics, and the substitute
    ///   does not, meaning that is is a [`Single`] wit no generics, we copy the
    ///   literal and retain our own generics, and recursively transverse those.
    ///
    /// # Failure
    ///
    /// - In all other cases, we give an error, as this would result in a
    ///   nonsensical generic.
    ///
    /// [`GenericTypeName`]:
    /// type_checker.context.generic_type_name.GenericTypeName.html
    /// [`Single`]:
    /// type_checker.context.generic_type_name.GenericTypeName.Single.html
    /// [`HashMap`]: std.collections.HashMap.html
    pub fn try_from(
        gen_type_name: &GenericTypeName,
        generics: &HashMap<String, GenericTypeName>,
        pos: &Position
    ) -> Result<TypeName, TypeErr> {
        let typename_from = |g| TypeName::try_from(g, generics, pos);
        match gen_type_name {
            GenericTypeName::Single { lit, generics: this_gens } => {
                if let Some(subst) = generics.get(lit) {
                    substitute(gen_type_name, subst, generics, pos)
                } else {
                    Ok(TypeName::Single {
                        lit:      lit.clone(),
                        generics: this_gens.iter().map(typename_from).collect::<Result<_, _>>()?
                    })
                }
            }
            GenericTypeName::Fun { args, ret_ty } => Ok(TypeName::Fun {
                args:   args.iter().map(typename_from).collect::<Result<_, _>>()?,
                ret_ty: Box::from(TypeName::try_from(ret_ty, generics, pos)?)
            }),
            GenericTypeName::Union { ty_names } => Ok(TypeName::Union {
                ty_names: ty_names.iter().map(typename_from).collect::<Result<_, _>>()?
            }),
            GenericTypeName::Tuple { ty_names } => Ok(TypeName::Tuple {
                ty_names: ty_names.iter().map(typename_from).collect::<Result<_, _>>()?
            })
        }
    }

    pub fn single(lit: &str, generics: &[TypeName]) -> TypeName {
        TypeName::Single { lit: String::from(lit), generics: Vec::from(generics) }
    }
}

fn substitute(
    this: &GenericTypeName,
    substitute: &GenericTypeName,
    generics: &HashMap<String, GenericTypeName>,
    pos: &Position
) -> Result<TypeName, TypeErr> {
    let typename_from = |g| TypeName::try_from(g, generics, pos);
    match (this, substitute) {
        (
            GenericTypeName::Single { generics: this_generics, .. },
            GenericTypeName::Single { lit: that_lit, generics: that_generics }
        ) => Ok(TypeName::Single {
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
        (GenericTypeName::Single { generics: this_generics, .. }, substitute)
            if this_generics.is_empty() =>
            TypeName::try_from(substitute, generics, pos),
        _ => Err(TypeErr::new(pos, "Unable to insert generic"))
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
            TypeName::Union { ty_names } => write!(f, "{{{}}}", comma_delimited(ty_names)?),
            TypeName::Tuple { ty_names } => write!(f, "({})", comma_delimited(ty_names)?)
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
