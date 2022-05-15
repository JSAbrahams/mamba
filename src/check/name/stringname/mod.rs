use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

use crate::check::context::{clss, Context, LookupClass};
use crate::check::context::clss::HasParent;
use crate::check::name::{CollectionType, IsSuperSet};
use crate::check::name::Name;
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
    pub generics: Vec<Name>,
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

impl CollectionType for StringName {
    fn collection_type(&self, ctx: &Context) -> TypeResult<Option<Name>> {
        if let Ok(clss) = ctx.class(self, &Position::default()) {
            // Must either return type as generic for matching, or check type (as here).
            let generics = &[Name::from(clss::INT_PRIMITIVE)];
            let col_string_name = StringName::new(clss::LIST, generics);

            let col_name = Name::from(&col_string_name);
            let parent = clss.has_parent(&col_name, ctx, &Position::default())?;
            return if parent {
                Ok(Some(Name::from(clss::INT_PRIMITIVE)))
            } else {
                Ok(None)
            };
        }
        Ok(None)
    }
}

impl From<&str> for StringName {
    fn from(name: &str) -> Self {
        StringName { name: String::from(name), generics: vec![] }
    }
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
            .flat_map(|n| other.generics.iter().map(move |o| n.is_superset_of(o, ctx, pos)))
            .collect::<Result<Vec<bool>, _>>()?
            .iter()
            .all(|b| *b))
    }
}

impl StringName {
    pub fn new(lit: &str, generics: &[Name]) -> StringName {
        StringName { name: String::from(lit), generics: Vec::from(generics) }
    }

    pub fn empty() -> StringName {
        StringName::new("()", &[])
    }

    pub fn substitute(
        &self,
        generics: &HashMap<Name, Name>,
        pos: &Position,
    ) -> TypeResult<StringName> {
        if let Some(name) = generics.get(&Name::from(self)) {
            let msg = format!("{} is not a DirectName", name);
            for string_name in name.as_direct(&msg, pos)? {
                return Ok(string_name);
            }

            let msg = format!("{} incorrect DirectName", name);
            Err(vec![TypeErr::new(pos, &msg)])
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
