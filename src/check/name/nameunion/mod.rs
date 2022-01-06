use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::IntoIter;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use crate::check::context::Context;
use crate::check::name::{AsMutable, AsNullable, IsNullable, IsSuperSet, Union};
use crate::check::name::stringname::StringName;
use crate::check::name::TrueName;
use crate::check::result::TypeResult;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod python;
pub mod generic;

#[derive(Debug, Clone, Eq)]
pub struct NameUnion {
    names: HashSet<TrueName>,
}

impl AsMutable for NameUnion {
    fn as_mutable(&self) -> Self {
        NameUnion { names: self.names.iter().map(|n| n.as_mutable()).collect() }
    }
}

impl Union<NameUnion> for NameUnion {
    fn union(&self, name: &NameUnion) -> Self {
        NameUnion { names: self.names.union(&name.names).cloned().collect() }
    }
}

impl Union<StringName> for NameUnion {
    fn union(&self, name: &StringName) -> Self {
        let mut names = self.names.clone();
        names.insert(TrueName::from(name));
        NameUnion { names }
    }
}

impl Union<TrueName> for NameUnion {
    fn union(&self, name: &TrueName) -> Self {
        let mut names = self.names.clone();
        names.insert(name.clone());
        NameUnion { names }
    }
}

impl From<&HashSet<NameUnion>> for NameUnion {
    fn from(names: &HashSet<NameUnion, RandomState>) -> Self {
        if let Some(mut name_union) = names.iter().last().cloned() {
            name_union.names().for_each(|name| name_union = name_union.union(&name));
            name_union
        } else {
            NameUnion::empty()
        }
    }
}

impl From<&StringName> for NameUnion {
    fn from(name: &StringName) -> Self {
        NameUnion { names: HashSet::from_iter(vec![TrueName::from(name)]) }
    }
}

impl PartialEq for NameUnion {
    fn eq(&self, other: &Self) -> bool {
        self.names.len() == other.names.len()
            && self.names.iter().zip(&other.names).all(|(this, that)| this == that)
    }
}

impl Hash for NameUnion {
    fn hash<H: Hasher>(&self, state: &mut H) { self.names().for_each(|n| n.hash(state)) }
}

impl Display for NameUnion {
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

impl From<&TrueName> for NameUnion {
    fn from(name: &TrueName) -> Self {
        let names: HashSet<TrueName> = HashSet::from_iter(vec![name.clone()]);
        NameUnion { names }
    }
}

impl From<&str> for NameUnion {
    fn from(name: &str) -> Self { NameUnion::from(&TrueName::from(name)) }
}

impl IsSuperSet<NameUnion> for NameUnion {
    fn is_superset_of(&self, other: &NameUnion, ctx: &Context, pos: &Position) -> TypeResult<bool> {
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

impl IsNullable for NameUnion {
    fn is_nullable(&self) -> bool { self.names.iter().all(|n| n.is_nullable()) }
}

impl AsNullable for NameUnion {
    fn as_nullable(&self) -> Self {
        NameUnion { names: self.names.iter().map(|n| n.as_nullable()).collect() }
    }
}

impl NameUnion {
    pub fn new(names: &[TrueName]) -> NameUnion {
        let names: HashSet<TrueName> = HashSet::from_iter(Vec::from(names));
        NameUnion { names }
    }

    pub fn is_empty(&self) -> bool { self == &NameUnion::empty() }

    pub fn as_direct(&self, msg: &str, pos: &Position) -> TypeResult<HashSet<StringName>> {
        self.names.iter().map(|n| n.as_direct(msg, pos)).collect::<Result<_, _>>()
    }

    pub fn empty() -> NameUnion { NameUnion { names: HashSet::new() } }

    pub fn is_null(&self) -> bool {
        self.names.iter().all(|name| name.is_null())
    }

    pub fn names(&self) -> IntoIter<TrueName> { self.names.clone().into_iter() }

    pub fn substitute(
        &self,
        generics: &HashMap<String, TrueName>,
        pos: &Position,
    ) -> TypeResult<NameUnion> {
        let names =
            self.names.iter().map(|n| n.substitute(generics, pos)).collect::<Result<_, _>>()?;
        Ok(NameUnion { names })
    }
}

#[cfg(test)]
mod tests {
    use crate::check::context::clss::{BOOL_PRIMITIVE, FLOAT_PRIMITIVE, INT_PRIMITIVE, STRING_PRIMITIVE};
    use crate::check::context::Context;
    use crate::check::name::IsSuperSet;
    use crate::check::name::nameunion::NameUnion;
    use crate::check::name::truename::TrueName;
    use crate::common::position::Position;

    #[test]
    fn test_is_superset_numbers() {
        let names = vec![
            TrueName::from(BOOL_PRIMITIVE),
            TrueName::from(STRING_PRIMITIVE),
            TrueName::from(INT_PRIMITIVE),
            TrueName::from(FLOAT_PRIMITIVE)];
        let union_1 = NameUnion::new(&names);
        let union_2 = NameUnion::from(INT_PRIMITIVE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, &Position::default()).unwrap())
    }

    #[test]
    fn test_is_superset_does_not_contain() {
        let names = vec![
            TrueName::from(BOOL_PRIMITIVE),
            TrueName::from(STRING_PRIMITIVE)];
        let union_1 = NameUnion::new(&names);
        let union_2 = NameUnion::from(INT_PRIMITIVE);

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
        let union_1 = NameUnion::new(&names);
        let union_2 = NameUnion::from(INT_PRIMITIVE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_2.is_superset_of(&union_1, &ctx, &Position::default()).unwrap())
    }
}
