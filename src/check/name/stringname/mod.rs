use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

use crate::check::context::{Context, LookupClass};
use crate::check::context::clss::HasParent;
use crate::check::name::IsSuperSet;
use crate::check::name::NameUnion;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod generic;

/// A direct truename is a string with accompanying generics.
///
/// Useful to denote class and function names, where Tuples and Anonymous
/// functions are not permitted.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringName {
    pub name: String,
    pub generics: Vec<NameUnion>,
}

impl Display for StringName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let generics = if self.generics.is_empty() {
            String::new()
        } else {
            format!("[{}]", comma_delm(&self.generics))
        };
        write!(f, "{}{}", self.name, generics)
    }
}

impl From<&str> for StringName {
    fn from(name: &str) -> Self { StringName { name: String::from(name), generics: vec![] } }
}

impl IsSuperSet<StringName> for StringName {
    fn is_superset_of(
        &self,
        other: &StringName,
        ctx: &Context,
        pos: &Position,
    ) -> TypeResult<bool> {
        Ok(ctx.class(other, pos)?.has_parent(self, ctx, pos)?
            && self
            .generics
            .iter()
            .map(|n| other.generics.iter().map(move |o| n.is_superset_of(o, ctx, pos)))
            .flatten()
            .collect::<Result<Vec<bool>, _>>()?
            .iter()
            .all(|b| *b))
    }
}

impl StringName {
    pub fn new(lit: &str, generics: &[NameUnion]) -> StringName {
        StringName { name: String::from(lit), generics: Vec::from(generics) }
    }

    pub fn empty() -> StringName { StringName::new("()", &[]) }

    pub fn substitute(
        &self,
        generics: &HashMap<String, TrueName>,
        pos: &Position,
    ) -> TypeResult<StringName> {
        if let Some(name) = generics.get(&self.name) {
            match &name.variant {
                NameVariant::Single(direct_name) if direct_name.generics.is_empty() =>
                    Ok(direct_name.clone()),
                _ => {
                    let msg = format!("Cannot substitute '{}' with `{}`", name.variant, name);
                    Err(vec![TypeErr::new(pos, &msg)])
                }
            }
        } else {
            Ok(StringName {
                name: self.name.clone(),
                generics: self
                    .generics
                    .iter()
                    .map(|generic| generic.substitute(generics, pos))
                    .collect::<Result<_, _>>()?,
            })
        }
    }
}
