use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;
use std::iter::FromIterator;

use crate::check::context::clss::NONE;
use crate::check::context::Context;
use crate::check::name::{ColType, Empty, IsSuperSet, Mutable, Name, Nullable, Substitute, Union};
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::result::{TypeResult, TypeTryFrom};
use crate::common::position::Position;

pub mod generic;
pub mod python;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrueName {
    is_nullable: bool,
    is_mutable: bool,
    pub variant: NameVariant,
}

impl PartialOrd<Self> for TrueName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.variant == other.variant {
            if self.is_nullable == other.is_nullable {
                self.is_mutable.partial_cmp(&other.is_mutable)
            } else {
                self.is_nullable.partial_cmp(&other.is_nullable)
            }
        } else {
            self.variant.partial_cmp(&other.variant)
        }
    }
}

impl Ord for TrueName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Mutable for TrueName {
    fn as_mutable(&self) -> Self { TrueName { is_mutable: true, ..self.clone() } }
}

impl From<&NameVariant> for TrueName {
    fn from(variant: &NameVariant) -> Self {
        TrueName { is_mutable: false, is_nullable: false, variant: variant.clone() }
    }
}

impl ColType for TrueName {
    fn col_type(&self, ctx: &Context, pos: Position) -> TypeResult<Option<Name>> {
        self.variant.col_type(ctx, pos)
    }
}

impl Display for TrueName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mutable = if self.is_mutable { "" } else { "fin " };
        write!(f, "{}{}{}", mutable, self.variant, if self.is_nullable { "?" } else { "" })
    }
}

impl From<&StringName> for TrueName {
    fn from(name: &StringName) -> Self {
        TrueName {
            is_nullable: false,
            is_mutable: false,
            variant: NameVariant::Single(name.clone()),
        }
    }
}

impl From<&str> for TrueName {
    fn from(name: &str) -> Self {
        TrueName {
            is_nullable: false,
            is_mutable: false,
            variant: NameVariant::Single(StringName::from(name)),
        }
    }
}

impl PartialEq<StringName> for TrueName {
    fn eq(&self, other: &StringName) -> bool {
        match &self.variant {
            NameVariant::Single(string_name) => string_name == other,
            _ => false
        }
    }
}

impl IsSuperSet<TrueName> for TrueName {
    /// Check if [TrueName] is supertype of other [TrueName].
    ///
    /// If self is nullable, then super of other iff:
    /// - Other is null.
    /// - Or, variant is supertype of other's variant. (Other may or may not be nullable.)
    /// If self is not nullable, then super of other iff:
    /// - Other is not nullable.
    /// - And, variant is supertype of other's variant.
    fn is_superset_of(&self, other: &TrueName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        if !self.is_empty() && other.is_empty() {
            return Ok(false);
        } else if self.is_nullable() && other.is_null() {
            return Ok(true); // Trivially true
        }

        let nullable_super = self.is_nullable() || (!self.is_nullable() && !other.is_nullable());
        Ok(nullable_super && self.variant.is_superset_of(&other.variant, ctx, pos)?)
    }
}

impl Union<TrueName> for Name {
    fn union(&self, name: &TrueName) -> Self {
        let mut names = self.names.clone();
        names.insert(name.clone());
        Name { names }
    }
}

impl Empty for TrueName {
    fn is_empty(&self) -> bool { self == &TrueName::empty() }
    fn empty() -> TrueName { TrueName::from(&StringName::empty()) }
}

impl Nullable for TrueName {
    fn is_nullable(&self) -> bool { self.is_nullable }
    fn is_null(&self) -> bool {
        matches!(&self.variant, NameVariant::Single(StringName { name, .. }) if name.clone() == *NONE)
    }
    fn as_nullable(&self) -> Self { TrueName { is_nullable: true, ..self.clone() } }
}

impl Substitute for TrueName {
    fn substitute(&self, generics: &HashMap<Name, Name>, pos: Position) -> TypeResult<TrueName> {
        let variant = match &self.variant {
            NameVariant::Single(direct_name) =>
                NameVariant::Single(direct_name.substitute(generics, pos)?),
            NameVariant::Tuple(names) => {
                let elements =
                    names.iter().map(|n| n.substitute(generics, pos)).collect::<Result<_, _>>()?;
                NameVariant::Tuple(elements)
            }
            NameVariant::Fun(args, ret) => NameVariant::Fun(
                args.iter().map(|a| a.substitute(generics, pos)).collect::<Result<_, _>>()?,
                Box::from(ret.substitute(generics, pos)?),
            )
        };

        Ok(TrueName { variant, ..self.clone() })
    }
}

impl TypeTryFrom<&TrueName> for StringName {
    fn try_from_pos(value: &TrueName, pos: Position) -> TypeResult<Self> {
        StringName::try_from_pos(&value.variant, pos)
    }
}

impl From<&Vec<TrueName>> for Name {
    fn from(names: &Vec<TrueName>) -> Self {
        let names: HashSet<TrueName> = HashSet::from_iter(names.iter().cloned());
        Name { names }
    }
}

impl TrueName {
    pub fn new(lit: &str, generics: &[Name]) -> TrueName {
        TrueName::from(&StringName::new(lit, generics))
    }
}

#[cfg(test)]
mod test {
    use crate::check::context::clss::{BOOL, COMPLEX, INT, STRING};
    use crate::check::name::IsSuperSet;
    use crate::check::name::truename::TrueName;
    use crate::common::position::Position;
    use crate::Context;

    #[test]
    fn bool_not_super_of_int() {
        let name_1 = TrueName::from(BOOL);
        let name_2 = TrueName::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn string_not_super_of_int() {
        let name_1 = TrueName::from(STRING);
        let name_2 = TrueName::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn complex_super_of_int() {
        let name_1 = TrueName::from(COMPLEX);
        let name_2 = TrueName::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }
}
