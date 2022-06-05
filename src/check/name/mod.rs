use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use itertools::Itertools;

use crate::check::context::Context;
use crate::check::ident::Identifier;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult, TypeTryFrom};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod namevariant;
pub mod stringname;
pub mod truename;

pub mod generic;
pub mod python;

pub const TEMP: char = '@';

pub trait Union<T> {
    fn union(&self, value: &T) -> Self;
}

pub trait IsSuperSet<T> {
    fn is_superset_of(&self, other: &T, ctx: &Context, pos: Position) -> TypeResult<bool>;
}

pub trait Empty {
    fn is_empty(&self) -> bool;
    fn empty() -> Self;
}

pub trait Nullable {
    fn is_nullable(&self) -> bool;
    fn is_null(&self) -> bool;
    fn as_nullable(&self) -> Self;
}

pub trait Mutable {
    fn as_mutable(&self) -> Self;
}

pub trait Substitute {
    fn substitute(&self, generics: &HashMap<Name, Name>, pos: Position) -> TypeResult<Self> where Self: Sized;
}

pub trait ColType {
    fn col_type(&self, ctx: &Context, pos: Position) -> TypeResult<Option<Name>>;
}

#[derive(Debug, Clone, Eq)]
pub struct Name {
    pub names: HashSet<TrueName>,
}

pub fn match_name(
    identifier: &Identifier,
    name: &Name,
    pos: Position,
) -> TypeResult<HashMap<String, (bool, Name)>> {
    let unions: Vec<HashMap<String, (bool, Name)>> =
        name.names.iter().map(|ty| match_type_direct(identifier, &ty, pos)).collect::<Result<_, _>>()?;

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

pub fn match_type_direct(
    identifier: &Identifier,
    name: &TrueName,
    pos: Position,
) -> TypeResult<HashMap<String, (bool, Name)>> {
    match &name.variant {
        NameVariant::Single { .. } | NameVariant::Fun { .. } => {
            if let Identifier::Single(mutable, id) = &identifier {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone().object(pos)?, (*mutable, Name::from(name)));
                Ok(mapping)
            } else {
                let msg = format!("Cannot match {} with a '{}'", identifier, name);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
        NameVariant::Tuple(elements) => match identifier {
            Identifier::Single(mutable, id) => {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone().object(pos)?, (*mutable, Name::from(name)));
                Ok(mapping)
            }
            Identifier::Multi(fields) if elements.len() == fields.len() => {
                let sets: Vec<HashMap<_, _>> = fields
                    .iter()
                    .zip(elements)
                    .map(|(identifier, ty)| match_name(identifier, ty, pos))
                    .collect::<Result<_, _>>()?;
                Ok(sets.into_iter().flatten().collect())
            }
            Identifier::Multi(idens) => {
                let msg = format!("Expected tuple of {}, was {}", elements.len(), idens.len());
                Err(vec![TypeErr::new(pos, &msg)])
            }
        },
    }
}

impl PartialOrd<Self> for Name {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_vec = self.names.iter().sorted();
        let other_vec = other.names.iter().sorted();
        self_vec.partial_cmp(other_vec)
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Mutable for Name {
    fn as_mutable(&self) -> Self {
        Name { names: self.names.iter().map(|n| n.as_mutable()).collect() }
    }
}

impl Union<Name> for Name {
    fn union(&self, name: &Name) -> Self {
        Name { names: self.names.union(&name.names).cloned().collect() }
    }
}

impl ColType for Name {
    fn col_type(&self, ctx: &Context, pos: Position) -> TypeResult<Option<Name>> {
        let names: Vec<Option<Name>> =
            self.names.iter().map(|n| n.col_type(ctx, pos)).collect::<Result<_, _>>()?;
        let mut union = Name::empty();
        for name in names {
            if let Some(name) = name {
                union = union.union(&name)
            } else {
                return Ok(None);
            }
        }
        Ok(Some(union))
    }
}

impl From<&HashSet<&str>> for Name {
    fn from(names: &HashSet<&str, RandomState>) -> Self {
        let names: HashSet<Name> = names.iter().map(|name| Name::from(*name)).collect();
        Name::from(&names)
    }
}

impl From<&HashSet<Name>> for Name {
    fn from(names: &HashSet<Name, RandomState>) -> Self {
        let mut final_name = Name::empty();
        for name in names {
            final_name = final_name.union(name);
        }
        final_name
    }
}

impl PartialEq<Self> for Name {
    fn eq(&self, other: &Self) -> bool {
        self.names.len() == other.names.len()
            && self.names.iter().all(|name| other.names.contains(name))
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.names.len().hash(state);
        self.names.iter().for_each(|n| n.hash(state))
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(first) = &self.names.iter().last() {
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
    fn from(name: &str) -> Self {
        Name::from(&TrueName::from(name))
    }
}

impl IsSuperSet<Name> for Name {
    fn is_superset_of(&self, other: &Name, ctx: &Context, pos: Position) -> TypeResult<bool> {
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

impl Nullable for Name {
    fn is_nullable(&self) -> bool {
        self.names.iter().all(|n| n.is_nullable())
    }

    fn is_null(&self) -> bool {
        self.names.iter().all(|name| name.is_null())
    }

    fn as_nullable(&self) -> Self {
        Name { names: self.names.iter().map(|n| n.as_nullable()).collect() }
    }
}

impl Empty for Name {
    fn is_empty(&self) -> bool {
        self == &Name::empty()
    }

    fn empty() -> Name {
        Name { names: HashSet::new() }
    }
}

impl Substitute for Name {
    fn substitute(&self, generics: &HashMap<Name, Name>, pos: Position) -> TypeResult<Name> {
        let names =
            self.names.iter().map(|n| n.substitute(generics, pos)).collect::<Result<_, _>>()?;
        Ok(Name { names })
    }
}

impl Name {
    pub fn as_direct(&self, pos: Position) -> TypeResult<HashSet<StringName>> {
        self.names.iter().map(|n| StringName::try_from_pos(n, pos)).collect::<Result<_, _>>()
    }

    pub fn contains(&self, item: &TrueName) -> bool {
        self.names.contains(item)
    }

    /// True if this was a temporary name, which is a name which starts with '@'.
    pub fn is_temporary(&self) -> bool {
        if let Some(name) = Vec::from_iter(&self.names).first() {
            match &name.variant {
                NameVariant::Single(stringname) => stringname.name.starts_with(TEMP),
                _ => false,
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::check::context::{clss, Context, LookupClass};
    use crate::check::context::clss::{BOOL, FLOAT, HasParent, INT, STRING, TUPLE};
    use crate::check::ident::Identifier;
    use crate::check::name::{ColType, Empty, IsSuperSet, match_name, Name, Nullable, Union};
    use crate::check::name::namevariant::NameVariant;
    use crate::check::name::stringname::StringName;
    use crate::check::name::truename::TrueName;
    use crate::check::result::TypeResult;
    use crate::common::position::Position;

    #[test]
    fn test_is_superset_numbers() {
        let names = vec![
            TrueName::from(BOOL),
            TrueName::from(STRING),
            TrueName::from(INT),
            TrueName::from(FLOAT),
        ];
        let union_1 = Name::from(&names);
        let union_2 = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn test_is_superset_slice_int() {
        let name1 = Name::from(&HashSet::from([clss::INT, clss::SLICE]));
        let name2 = Name::from(&HashSet::from([clss::INT]));

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(name1.is_superset_of(&name2, &ctx, Position::default()).unwrap());
        assert!(!name2.is_superset_of(&name1, &ctx, Position::default()).unwrap());
    }

    #[test]
    fn is_superset_does_not_contain() {
        let union_1 = Name::from(&vec![TrueName::from(BOOL), TrueName::from(STRING)]);
        let union_2 = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_1.is_superset_of(&union_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn is_superset_contains() {
        let union_1 = Name::from(&vec![TrueName::from(BOOL), TrueName::from(INT)]);
        let union_2 = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn nullabel_int_superset_int() {
        let true_name = TrueName::from(INT).as_nullable();
        let union_1 = Name::from(&true_name);
        let union_2 = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn int_not_superset_nullable_int() {
        let true_name = TrueName::from(INT).as_nullable();
        let union_1 = Name::from(INT);
        let union_2 = Name::from(&true_name);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_1.is_superset_of(&union_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn is_superset_self() {
        let (name_1, name_2) = (Name::from(TUPLE), Name::from(TUPLE));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn int_not_superset_string() {
        let (name_1, name_2) = (Name::from(INT), Name::from(STRING));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn test_temp_name() {
        let name1 = Name::from("@something");
        let name2 = Name::from(clss::INT);

        assert!(name1.is_temporary());
        assert!(!name2.is_temporary())
    }

    #[test]
    fn test_superset_wrong_way() {
        let names = vec![
            TrueName::from(BOOL),
            TrueName::from(STRING),
            TrueName::from(INT),
            TrueName::from(FLOAT),
        ];
        let union_1 = Name::from(&names);
        let union_2 = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_2.is_superset_of(&union_1, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn test_name_equal() {
        let name1 = Name::from("MyType");
        let name2 = Name::from("MyType");
        assert_eq!(name1, name2);
    }

    #[test]
    fn test_name_not_equal() {
        let name1 = Name::from("MyType");
        let name2 = Name::from("MyType2");
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_name_is_nullable() {
        let name1 = Name::from("MyType").as_nullable();
        assert!(name1.is_nullable());
    }

    #[test]
    fn test_name_is_nullable_is_not_null() {
        let name1 = Name::from("MyType").as_nullable();
        assert!(!name1.is_null());
    }

    #[test]
    fn test_name_none_is_null() {
        let name1 = Name::from(clss::NONE);
        assert!(name1.is_null());
    }

    #[test]
    fn test_name_none_is_not_nullable() {
        let name1 = Name::from(clss::NONE);
        assert!(!name1.is_nullable());
    }

    #[test]
    fn test_name_nullable_not_equal() {
        let name1 = Name::from("MyType").as_nullable();
        let name2 = Name::from("MyType");
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_tuple_equal() {
        let name1 = Name::from(&NameVariant::Tuple(vec![Name::from("A1"), Name::from("A2")]));
        let name2 = Name::from(&NameVariant::Tuple(vec![Name::from("A1"), Name::from("A2")]));
        assert_eq!(name1, name2)
    }

    #[test]
    fn test_tuple_not_equal_size() {
        let name1 = Name::from(&NameVariant::Tuple(vec![Name::from("A1"), Name::from("A2")]));
        let name2 = Name::from(&NameVariant::Tuple(vec![
            Name::from("A1"),
            Name::from("A2"),
            Name::from("A2"),
        ]));
        assert_ne!(name1, name2)
    }

    #[test]
    fn test_tuple_not_equal() {
        let name1 = Name::from(&NameVariant::Tuple(vec![Name::from("A1"), Name::from("A2")]));
        let name2 = Name::from(&NameVariant::Tuple(vec![Name::from("A2"), Name::from("A1")]));
        assert_ne!(name1, name2)
    }

    #[test]
    fn test_match_name() -> TypeResult<()> {
        let iden = Identifier::from((false, "abc"));
        let name = Name::from("MyType3");
        let matchings = match_name(&iden, &name, Position::default())?;

        assert_eq!(matchings.len(), 1);
        let (mutable, matching) = matchings["abc"].clone();
        assert!(!mutable);
        assert_eq!(matching, name);

        Ok(())
    }

    #[test]
    fn test_match_name_err_iden_is_tuple() {
        let iden = Identifier::from((false, "abc"));
        let iden2 = Identifier::from((false, "abc2"));
        let iden3 = Identifier::from(&vec![iden, iden2]);

        let name = Name::from("MyType3");
        let err = match_name(&iden3, &name, Position::default());

        assert!(err.is_err())
    }

    #[test]
    fn test_match_name_type_is_tuple() -> TypeResult<()> {
        let iden = Identifier::from((false, "abc"));
        let name = Name::from(&NameVariant::Tuple(vec![Name::from("A2"), Name::from("A1")]));
        let matchings = match_name(&iden, &name, Position::default())?;

        assert_eq!(matchings.len(), 1);
        let (mutable, matching) = matchings["abc"].clone();
        assert!(!mutable);
        assert_eq!(matching, name);

        Ok(())
    }

    #[test]
    fn test_match_name_iden_and_type_is_tuple() -> TypeResult<()> {
        let iden = Identifier::from((false, "abc"));
        let iden2 = Identifier::from((false, "abc2"));
        let iden3 = Identifier::from(&vec![iden, iden2]);

        let name = Name::from(&NameVariant::Tuple(vec![Name::from("A2"), Name::from("A1")]));

        let matchings = match_name(&iden3, &name, Position::default())?;

        assert_eq!(matchings.len(), 2);
        let (mutable1, matching1) = matchings["abc"].clone();
        let (mutable2, matching2) = matchings["abc2"].clone();

        assert!(!mutable1);
        assert!(!mutable2);
        assert_eq!(matching1, Name::from("A2"));
        assert_eq!(matching2, Name::from("A1"));

        Ok(())
    }

    #[test]
    fn test_match_name_iden_wrong_size_and_type() {
        let iden = Identifier::from((false, "abc"));
        let iden2 = Identifier::from((false, "abc2"));
        let iden3 = Identifier::from((false, "abc2"));
        let iden4 = Identifier::from(&vec![iden, iden2, iden3]);

        let name = Name::from(&NameVariant::Tuple(vec![Name::from("A2"), Name::from("A1")]));

        let matchings = match_name(&iden4, &name, Position::default());
        assert!(matchings.is_err());
    }

    #[test]
    fn test_match_name_iden_and_type_wrong_size() {
        let iden = Identifier::from((false, "abc"));
        let iden2 = Identifier::from((false, "abc2"));
        let iden3 = Identifier::from(&vec![iden, iden2]);

        let name = Name::from(&NameVariant::Tuple(vec![
            Name::from("A2"),
            Name::from("A1"),
            Name::from("A0"),
        ]));

        let matchings = match_name(&iden3, &name, Position::default());
        assert!(matchings.is_err());
    }

    #[test]
    fn range_has_collection_int_as_parent() -> TypeResult<()> {
        let range_name = Name::from(clss::RANGE);
        let int_name = Name::from(clss::INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        let collection_ty = range_name.col_type(&ctx, Position::default())?;
        assert_eq!(collection_ty, Some(int_name));
        Ok(())
    }

    #[test]
    fn float_parent_of_int() -> TypeResult<()> {
        let float_name = Name::from(clss::FLOAT);
        let int_name = StringName::from(clss::INT);
        let ctx = Context::default().into_with_primitives().unwrap();

        let clss = ctx.class(&int_name, Position::default())?;
        assert!(clss.has_parent(&float_name, &ctx, Position::default())?);
        Ok(())
    }

    #[test]
    fn int_not_parent_of_float() -> TypeResult<()> {
        let float_name = StringName::from(clss::FLOAT);
        let int_name = Name::from(clss::INT);
        let ctx = Context::default().into_with_primitives().unwrap();

        let clss = ctx.class(&float_name, Position::default())?;
        assert!(!clss.has_parent(&int_name, &ctx, Position::default())?);
        Ok(())
    }

    #[test]
    fn slice_not_collection_int_as_parent() {
        let range_name = Name::from(clss::SLICE);

        let ctx = Context::default().into_with_primitives().unwrap();
        let collection_ty = range_name.col_type(&ctx, Position::default());
        assert!(collection_ty.is_err());
    }

    #[test]
    fn name_fold() {
        let int_name = Name::from(clss::INT);
        let float_name = Name::from(clss::FLOAT);

        let name1 = Name::from(&HashSet::from([int_name.clone(), float_name.clone()]));
        let name2 = [int_name, float_name].iter().fold(Name::empty(), |name, n| name.union(n));
        assert_eq!(name1, name2);
    }
}
