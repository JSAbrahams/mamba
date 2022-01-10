use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::IntoIter;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use crate::check::context::Context;
use crate::check::ident::Identifier;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod stringname;
pub mod truename;
pub mod namevariant;

pub mod generic;
pub mod python;

pub trait Union<T> {
    #[must_use]
    fn union(&self, value: &T) -> Self;
}

pub trait IsSuperSet<T> {
    fn is_superset_of(&self, other: &T, ctx: &Context, pos: &Position) -> TypeResult<bool>;
}

pub trait IsNullable {
    fn is_nullable(&self) -> bool;
}

pub trait AsNullable {
    #[must_use]
    fn as_nullable(&self) -> Self;
}

pub trait AsMutable {
    #[must_use]
    fn as_mutable(&self) -> Self;
}

pub fn match_name(identifier: &Identifier, name: &Name, pos: &Position) -> TypeResult<HashMap<String, (bool, Name)>> {
    let unions: Vec<HashMap<String, (bool, Name)>> =
        name.names().map(|ty| match_type_direct(identifier, &ty, pos)).collect::<Result<_, _>>()?;

    let mut final_union: HashMap<String, (bool, Name)> = HashMap::new();
    for union in unions {
        for (id, (mutable, name)) in union {
            if let Some((current_mutable, current_name)) =
            final_union.insert(id.clone(), (mutable, name.clone()))
            {
                final_union
                    .insert(id.clone(), (mutable && current_mutable, current_name.union(&name)));
            }
        }
    }

    Ok(final_union)
}

pub fn match_type_direct(identifier: &Identifier, name: &TrueName, pos: &Position) -> TypeResult<HashMap<String, (bool, Name)>> {
    match &name.variant {
        NameVariant::Single { .. } | NameVariant::Fun { .. } =>
            if let Some((mutable, id)) = &identifier.lit {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone(), (*mutable, Name::from(name)));
                Ok(mapping)
            } else {
                let msg = format!("Cannot match {} with a '{}'", identifier, name);
                Err(vec![TypeErr::new(pos, &msg)])
            },
        NameVariant::Tuple(elements) =>
            if let Some((mutable, id)) = &identifier.lit {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone(), (*mutable, Name::from(name)));
                Ok(mapping)
            } else if elements.len() == identifier.fields().len() {
                let sets: Vec<HashMap<_, _>> = identifier
                    .names
                    .iter()
                    .zip(elements)
                    .map(|(identifier, ty)| match_name(identifier, ty, pos))
                    .collect::<Result<_, _>>()?;

                Ok(sets.into_iter().flatten().collect())
            } else {
                let msg = format!(
                    "Expected tuple of {}, but was {}.",
                    identifier.fields().len(),
                    elements.len()
                );
                Err(vec![TypeErr::new(pos, &msg)])
            },
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Name {
    names: HashSet<TrueName>,
}

impl AsMutable for Name {
    fn as_mutable(&self) -> Self {
        Name { names: self.names.iter().map(|n| n.as_mutable()).collect() }
    }
}

impl Union<Name> for Name {
    fn union(&self, name: &Name) -> Self {
        Name { names: self.names.union(&name.names).cloned().collect() }
    }
}

impl Union<StringName> for Name {
    fn union(&self, name: &StringName) -> Self {
        let mut names = self.names.clone();
        names.insert(TrueName::from(name));
        Name { names }
    }
}

impl Union<TrueName> for Name {
    fn union(&self, name: &TrueName) -> Self {
        let mut names = self.names.clone();
        names.insert(name.clone());
        Name { names }
    }
}

impl From<&HashSet<Name>> for Name {
    fn from(names: &HashSet<Name, RandomState>) -> Self {
        if let Some(mut name_union) = names.iter().last().cloned() {
            name_union.names().for_each(|name| name_union = name_union.union(&name));
            name_union
        } else {
            Name::empty()
        }
    }
}

impl From<&StringName> for Name {
    fn from(name: &StringName) -> Self {
        Name { names: HashSet::from_iter(vec![TrueName::from(name)]) }
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.names.len() == other.names.len()
            && self.names.iter().zip(&other.names).all(|(this, that)| this == that)
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) { self.names().for_each(|n| n.hash(state)) }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(first) = &self.names().last() {
            if self.names.len() > 1 {
                write!(f, "{{{}}}", comma_delm(&self.names))
            } else {
                write!(f, "{}", first)
            }
        } else {
            write!(f, "()")
        }
    }
}

impl From<&TrueName> for Name {
    fn from(name: &TrueName) -> Self {
        let names: HashSet<TrueName> = HashSet::from_iter(vec![name.clone()]);
        Name { names }
    }
}

impl From<&str> for Name {
    fn from(name: &str) -> Self { Name::from(&TrueName::from(name)) }
}

impl IsSuperSet<Name> for Name {
    fn is_superset_of(&self, other: &Name, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        if !self.is_empty() && other.is_empty() {
            return Ok(false);
        }

        for name in &other.names {
            let is_superset = |s_name: &TrueName| s_name.is_superset_of(name, ctx, pos);
            let any_superset: Vec<bool> =
                self.names.iter().map(is_superset).collect::<Result<_, _>>()?;
            if !any_superset.iter().any(|b| *b) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl IsNullable for Name {
    fn is_nullable(&self) -> bool { self.names.iter().all(|n| n.is_nullable()) }
}

impl AsNullable for Name {
    fn as_nullable(&self) -> Self {
        Name { names: self.names.iter().map(|n| n.as_nullable()).collect() }
    }
}

impl Name {
    pub fn new(names: &[TrueName]) -> Name {
        let names: HashSet<TrueName> = HashSet::from_iter(Vec::from(names));
        Name { names }
    }

    pub fn is_empty(&self) -> bool { self == &Name::empty() }

    pub fn as_direct(&self, msg: &str, pos: &Position) -> TypeResult<HashSet<StringName>> {
        self.names.iter().map(|n| n.as_direct(msg, pos)).collect::<Result<_, _>>()
    }

    pub fn empty() -> Name { Name { names: HashSet::new() } }

    pub fn is_null(&self) -> bool {
        self.names.iter().all(|name| name.is_null())
    }

    pub fn names(&self) -> IntoIter<TrueName> { self.names.clone().into_iter() }

    pub fn substitute(
        &self,
        generics: &HashMap<String, TrueName>,
        pos: &Position,
    ) -> TypeResult<Name> {
        let names =
            self.names.iter().map(|n| n.substitute(generics, pos)).collect::<Result<_, _>>()?;
        Ok(Name { names })
    }
}

#[cfg(test)]
mod tests {
    use crate::check::context::clss::{BOOL_PRIMITIVE, FLOAT_PRIMITIVE, INT_PRIMITIVE, STRING_PRIMITIVE};
    use crate::check::context::Context;
    use crate::check::name::IsSuperSet;
    use crate::check::name::Name;
    use crate::check::name::truename::TrueName;
    use crate::common::position::Position;

    #[test]
    fn test_is_superset_numbers() {
        let names = vec![
            TrueName::from(BOOL_PRIMITIVE),
            TrueName::from(STRING_PRIMITIVE),
            TrueName::from(INT_PRIMITIVE),
            TrueName::from(FLOAT_PRIMITIVE)];
        let union_1 = Name::new(&names);
        let union_2 = Name::from(INT_PRIMITIVE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, &Position::default()).unwrap())
    }

    #[test]
    fn test_is_superset_does_not_contain() {
        let names = vec![
            TrueName::from(BOOL_PRIMITIVE),
            TrueName::from(STRING_PRIMITIVE)];
        let union_1 = Name::new(&names);
        let union_2 = Name::from(INT_PRIMITIVE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_1.is_superset_of(&union_2, &ctx, &Position::default()).unwrap())
    }

    #[test]
    fn test_superset_wrong_way() {
        let names = vec![
            TrueName::from(BOOL_PRIMITIVE),
            TrueName::from(STRING_PRIMITIVE),
            TrueName::from(INT_PRIMITIVE),
            TrueName::from(FLOAT_PRIMITIVE)];
        let union_1 = Name::new(&names);
        let union_2 = Name::from(INT_PRIMITIVE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_2.is_superset_of(&union_1, &ctx, &Position::default()).unwrap())
    }
}
