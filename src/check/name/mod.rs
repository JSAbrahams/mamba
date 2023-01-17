use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use itertools::Itertools;

use crate::check::context::{clss, Context};
use crate::check::context::clss::Class;
use crate::check::ident::Identifier;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::{IsTemp, MatchTempName, TrueName};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod string_name;
pub mod true_name;

pub mod generic;
pub mod python;

pub const TEMP: char = '@';

pub type NameMap = HashMap<Name, Name>;

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

pub trait Any {
    fn any() -> Self;
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
    fn substitute(&self, generics: &NameMap, pos: Position) -> TypeResult<Self> where Self: Sized;
}

pub trait ColType {
    fn col_type(&self, ctx: &Context, pos: Position) -> TypeResult<Option<Name>>;
}

pub trait ContainsTemp {
    fn contains_temp(&self) -> bool;
}

pub trait TupleCallable<T1, T2, T3> {
    fn tuple(names: &[Name]) -> Self;
    fn callable(args: &[Name], ret_ty: &Name) -> Self;

    fn is_tuple(&self) -> T1;
    fn is_callable(&self) -> T1;

    fn elements(&self, pos: Position) -> TypeResult<T2>;
    fn args(&self, pos: Position) -> TypeResult<T2>;
    fn ret_ty(&self, pos: Position) -> TypeResult<T3>;
}

#[derive(Debug, Clone, Eq, Default)]
pub struct Name {
    pub names: HashSet<TrueName>,
    pub is_interchangeable: bool,
}

impl Any for Name {
    fn any() -> Self {
        Name::from(clss::ANY)
    }
}

pub fn match_name(
    identifier: &Identifier,
    name: &Name,
    pos: Position,
) -> TypeResult<HashMap<String, (bool, Name)>> {
    let unions: Vec<HashMap<String, (bool, Name)>> =
        name.names.iter().map(|ty| match_type_direct(identifier, ty, pos)).collect::<Result<_, _>>()?;

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
    if let Ok(elements) = name.elements(pos) {
        match identifier {
            Identifier::Single(mutable, id) => {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone().object(pos)?, (*mutable, Name::from(name)));
                Ok(mapping)
            }
            Identifier::Multi(fields) if elements.len() == fields.len() => {
                let sets: Vec<HashMap<_, _>> = fields
                    .iter()
                    .zip(elements)
                    .map(|(identifier, ty)| match_name(identifier, &ty, pos))
                    .collect::<Result<_, _>>()?;
                Ok(sets.into_iter().flatten().collect())
            }
            Identifier::Multi(idens) => {
                let msg = format!("Expected tuple of {}, was {}", elements.len(), idens.len());
                Err(vec![TypeErr::new(pos, &msg)])
            }
        }
    } else if let Identifier::Single(mutable, id) = &identifier {
        let mut mapping = HashMap::with_capacity(1);
        mapping.insert(id.clone().object(pos)?, (*mutable, Name::from(name)));
        Ok(mapping)
    } else {
        let msg = format!("Cannot match {identifier} with a '{name}'");
        Err(vec![TypeErr::new(pos, &msg)])
    }
}

impl From<&HashSet<Class>> for Name {
    fn from(classes: &HashSet<Class>) -> Self {
        Name { names: classes.iter().map(|c| TrueName::from(&c.name)).collect(), is_interchangeable: false }
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
        Name { names: self.names.iter().map(|n| n.as_mutable()).collect(), ..self.clone() }
    }
}

impl Union<Name> for Name {
    fn union(&self, name: &Name) -> Self {
        let names: HashSet<TrueName> = self.names.union(&name.names).cloned().collect();
        Name {
            names: if names.iter().any(TrueName::is_null) && names.len() > 1 {
                names.iter()
                    .filter(|n| !n.is_null())
                    .map(TrueName::as_nullable)
                    .collect()
            } else {
                names
            },
            is_interchangeable: self.is_interchangeable || name.is_interchangeable,
        }
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

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.names.eq(&other.names) // order doesn't matter for partialeq
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.names.len().hash(state);
        self.names.iter()
            .sorted_by_key(|name| &name.variant)
            .for_each(|n| n.hash(state))
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(first) = &self.names.iter().last() {
            if self.names.len() > 1 {
                write!(f, "{{{}}}", comma_delm(&self.names))
            } else {
                write!(f, "{first}")
            }
        } else {
            write!(f, "()")
        }
    }
}

impl From<&TrueName> for Name {
    fn from(name: &TrueName) -> Self {
        let names: HashSet<TrueName> = HashSet::from_iter(vec![name.clone()]);
        Name { names, is_interchangeable: false }
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

        let mut self_is_super_of = false;
        for name in &other.names {
            let is_superset = |s_name: &TrueName| s_name.is_superset_of(name, ctx, pos);
            let any_superset: Vec<_> = self.names.iter().map(is_superset).collect::<Result<_, _>>()?;

            if !other.is_interchangeable && any_superset.clone().iter().all(|b| !*b) {
                return Ok(false); // not a single of self was super
            }

            self_is_super_of |= any_superset.iter().any(|b| *b);
        }

        Ok(if other.is_interchangeable { self_is_super_of } else { true })
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
        Name { names: self.names.iter().map(|n| n.as_nullable()).collect(), ..self.clone() }
    }
}

impl Empty for Name {
    fn is_empty(&self) -> bool {
        self == &Name::empty() || self.names.iter().all(TrueName::is_empty)
    }

    fn empty() -> Name {
        Name { names: HashSet::new(), is_interchangeable: false }
    }
}

impl Substitute for Name {
    fn substitute(&self, generics: &HashMap<Name, Name>, pos: Position) -> TypeResult<Name> {
        let names =
            self.names.iter().map(|n| n.substitute(generics, pos)).collect::<Result<_, _>>()?;
        Ok(Name { names, ..self.clone() })
    }
}

impl ContainsTemp for Name {
    fn contains_temp(&self) -> bool {
        self.names.iter().any(TrueName::contains_temp)
    }
}

impl TupleCallable<HashSet<bool>, HashSet<Vec<Name>>, HashSet<Name>> for Name {
    fn tuple(names: &[Name]) -> Self {
        Self { names: HashSet::from([TrueName::tuple(names)]), ..Default::default() }
    }

    fn callable(args: &[Name], ret_ty: &Name) -> Self {
        Self { names: HashSet::from([TrueName::callable(args, ret_ty)]), ..Default::default() }
    }

    fn is_tuple(&self) -> HashSet<bool> {
        self.names.iter().map(|n| n.is_tuple()).collect()
    }

    fn is_callable(&self) -> HashSet<bool> {
        self.names.iter().map(|n| n.is_callable()).collect()
    }

    fn elements(&self, pos: Position) -> TypeResult<HashSet<Vec<Name>>> {
        self.names.iter().map(|n| n.elements(pos)).collect()
    }

    fn args(&self, pos: Position) -> TypeResult<HashSet<Vec<Name>>> {
        self.names.iter().map(|n| n.args(pos)).collect()
    }

    fn ret_ty(&self, pos: Position) -> TypeResult<HashSet<Name>> {
        self.names.iter().map(|n| n.ret_ty(pos)).collect()
    }
}

impl Name {
    pub fn trim_any(&self) -> Self {
        let names = self.names.iter().filter(|n| **n != TrueName::any()).cloned().collect();
        Name { names, ..self.clone() }
    }

    pub fn trim(&self, ty: &str) -> Self {
        let names = self.names.iter().flat_map(|n| n.trim(ty)).collect();
        Name { names, ..self.clone() }
    }

    /// Trim Name by removing any in the set which are superset of other members in the set.
    pub fn trim_super(&self, ctx: &Context) -> Self {
        let names = if self.names.len() > 1 {
            self.names
                .iter()
                .filter(|n| self.names.iter().any(|o_n| {
                    !o_n.is_superset_of(n, ctx, Position::invisible()).unwrap_or(true)
                }))
                .cloned()
                .collect()
        } else {
            self.names.clone()
        };
        Self { names, ..self.clone() }
    }

    pub fn as_direct(&self) -> HashSet<StringName> {
        self.names.iter().map(StringName::from).collect()
    }

    /// Any means that if one check if another [is_superset_of] self, then it will be true if it is
    /// just a superset of one.
    pub fn is_interchangeable(&self, is_interchangeable: bool) -> Self {
        Name { is_interchangeable, ..self.clone() }
    }

    pub fn contains(&self, item: &TrueName) -> bool {
        self.names.contains(item)
    }

    /// True if this was a temporary name, which is a name which starts with '@'.
    pub fn is_temporary(&self) -> bool {
        if let Some(name) = Vec::from_iter(&self.names).first() {
            return name.variant.is_temp();
        }
        false
    }

    pub fn as_name(&self, true_name: &TrueName, pos: Position) -> TypeResult<Name> {
        self.names.get(true_name).map(Name::from).ok_or_else(|| {
            let msg = format!("{self} does not define {true_name}");
            vec![TypeErr::new(pos, &msg)]
        })
    }

    pub fn temp_map(&self, other: &Name, pos: Position) -> TypeResult<HashMap<Name, Name>> {
        self.temp_map_with_mapping(other, HashMap::new(), pos)
    }

    pub(crate) fn temp_map_with_mapping(&self, other: &Name, mapping: NameMap, pos: Position) -> TypeResult<NameMap> {
        self.names.iter().fold(Ok(mapping), |acc, s_n| {
            other.names.iter().fold(acc, |acc, o_n| if let Ok(acc) = acc {
                s_n.temp_map(&o_n.variant, acc, pos)
            } else { acc })
        })
    }

    pub(crate) fn match_name_helper(&self, other: &Name, mapping: &mut NameMap, pos: Position) -> TypeResult<()> {
        for name in &self.names {
            for other_name in &other.names {
                name.variant.match_name_helper(&other_name.variant, mapping, pos)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::check::context::{clss, Context, LookupClass};
    use crate::check::context::clss::{BOOL, COLLECTION, FLOAT, HasParent, INT, RANGE, SET, STRING, TUPLE};
    use crate::check::ident::Identifier;
    use crate::check::name::{Any, ColType, Empty, IsSuperSet, match_name, Name, Nullable, TupleCallable, Union};
    use crate::check::name::string_name::StringName;
    use crate::check::name::true_name::TrueName;
    use crate::check::result::TypeResult;
    use crate::common::position::Position;

    #[test]
    fn trim_super_nullable() {
        let name_1 = Name::from("Str");
        let name_2 = Name::from(&TrueName::from("Str").as_nullable());

        let union = name_1.union(&name_2);
        assert_eq!(union, Name::from(&HashSet::from([name_1, name_2.clone()])));

        let ctx = Context::default().into_with_primitives().unwrap().into_with_std_lib().unwrap();
        let union = union.trim_super(&ctx);
        assert_eq!(union, Name::from(&HashSet::from([name_2])));
    }

    #[test]
    fn trim_super_float_int_is_float() {
        let name_1 = Name::from("Float");
        let name_2 = Name::from("Int");

        let union = name_1.union(&name_2);
        assert_eq!(union, Name::from(&HashSet::from([name_1.clone(), name_2])));

        let ctx = Context::default().into_with_primitives().unwrap().into_with_std_lib().unwrap();
        let union = union.trim_super(&ctx);
        assert_eq!(union, Name::from(&HashSet::from([name_1])));
    }

    #[test]
    fn trim_super_of_self() {
        let name_1 = Name::from("Float");
        let ctx = Context::default().into_with_primitives().unwrap().into_with_std_lib().unwrap();
        let union = name_1.trim_super(&ctx);
        assert_eq!(union, Name::from(&HashSet::from([name_1])));
    }

    #[test]
    fn union_none_nullable_str_is_nullable_str() {
        let name_1 = Name::from("None");
        let name_2 = Name::from(&TrueName::from("Str").as_nullable());

        let union = name_1.union(&name_2);
        assert_eq!(union.names.len(), 1);
        assert_eq!(*union.names.iter().next().unwrap(), TrueName::from("Str").as_nullable());
    }

    #[test]
    fn union_none_str_is_nullable_str() {
        let name_1 = Name::from("None");
        let name_2 = Name::from(&TrueName::from("Str"));

        let union = name_1.union(&name_2);
        assert_eq!(union.names.len(), 1);
        assert_eq!(*union.names.iter().next().unwrap(), TrueName::from("Str").as_nullable());
    }

    #[test]
    fn collect_any_superset_of_set_int() {
        let union_1 = Name::from(&StringName::new(COLLECTION, &[Name::any()]));
        let union_2 = Name::from(&StringName::new(SET, &[Name::from(INT)]));

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, Position::invisible()).unwrap())
    }

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
        assert!(union_1.is_superset_of(&union_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn test_is_superset_slice_int() {
        let name1 = Name::from(&HashSet::from([INT, clss::SLICE]));
        let name2 = Name::from(&HashSet::from([INT]));

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(name1.is_superset_of(&name2, &ctx, Position::invisible()).unwrap());
        assert!(!name2.is_superset_of(&name1, &ctx, Position::invisible()).unwrap());
    }

    #[test]
    fn is_superset_does_not_contain() {
        let union_1 = Name::from(&vec![TrueName::from(BOOL), TrueName::from(STRING)]);
        let union_2 = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_1.is_superset_of(&union_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn is_superset_contains() {
        let union_1 = Name::from(&vec![TrueName::from(BOOL), TrueName::from(INT)]);
        let union_2 = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn is_superset_any_only_one() {
        let union_1 = Name::from(&vec![TrueName::from(BOOL), TrueName::from(INT)]).is_interchangeable(true);
        let union_2 = Name::from(BOOL);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_2.is_superset_of(&union_1, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn is_superset_any_no_one() {
        let union_1 = Name::from(&vec![TrueName::from(BOOL), TrueName::from(INT)]).is_interchangeable(true);
        let union_2 = Name::from(RANGE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_2.is_superset_of(&union_1, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn is_superset_any_no_one_2() {
        let union_1 = Name::from(&vec![TrueName::from(INT)]).is_interchangeable(true);
        let union_2 = Name::from(STRING);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_2.is_superset_of(&union_1, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn is_superset_any_nullable() {
        let union_1 = Name::from(&vec![TrueName::from(BOOL).as_nullable(), TrueName::from(INT)]).is_interchangeable(true);
        let union_2 = Name::from(BOOL);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_2.is_superset_of(&union_1, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn nullabel_int_superset_int() {
        let true_name = TrueName::from(INT).as_nullable();
        let union_1 = Name::from(&true_name);
        let union_2 = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn int_not_superset_nullable_int() {
        let true_name = TrueName::from(INT).as_nullable();
        let union_1 = Name::from(INT);
        let union_2 = Name::from(&true_name);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_1.is_superset_of(&union_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn is_superset_self() {
        let (name_1, name_2) = (Name::from(TUPLE), Name::from(TUPLE));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(name_1.is_superset_of(&name_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn int_not_superset_string() {
        let (name_1, name_2) = (Name::from(INT), Name::from(STRING));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn test_temp_name() {
        let name1 = Name::from("@something");
        let name2 = Name::from(INT);

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
        assert!(!union_2.is_superset_of(&union_1, &ctx, Position::invisible()).unwrap())
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
        let name1 = Name::tuple(&[Name::from("A1"), Name::from("A2")]);
        let name2 = Name::tuple(&[Name::from("A1"), Name::from("A2")]);
        assert_eq!(name1, name2)
    }

    #[test]
    fn test_tuple_not_equal_size() {
        let name1 = Name::tuple(&[Name::from("A1"), Name::from("A2")]);
        let name2 = Name::tuple(&[
            Name::from("A1"),
            Name::from("A2"),
            Name::from("A2"),
        ]);
        assert_ne!(name1, name2)
    }

    #[test]
    fn test_tuple_not_equal() {
        let name1 = Name::tuple(&[Name::from("A1"), Name::from("A2")]);
        let name2 = Name::tuple(&[Name::from("A2"), Name::from("A1")]);
        assert_ne!(name1, name2)
    }

    #[test]
    fn test_match_name() -> TypeResult<()> {
        let iden = Identifier::from((false, "abc"));
        let name = Name::from("MyType3");
        let matchings = match_name(&iden, &name, Position::invisible())?;

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
        let err = match_name(&iden3, &name, Position::invisible());

        assert!(err.is_err())
    }

    #[test]
    fn test_match_name_type_is_tuple() -> TypeResult<()> {
        let iden = Identifier::from((false, "abc"));
        let name = Name::tuple(&[Name::from("A2"), Name::from("A1")]);
        let matchings = match_name(&iden, &name, Position::invisible())?;

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

        let name = Name::tuple(&[Name::from("A2"), Name::from("A1")]);

        let matchings = match_name(&iden3, &name, Position::invisible())?;

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

        let name = Name::tuple(&[Name::from("A2"), Name::from("A1")]);

        let matchings = match_name(&iden4, &name, Position::invisible());
        assert!(matchings.is_err());
    }

    #[test]
    fn test_match_name_iden_and_type_wrong_size() {
        let iden = Identifier::from((false, "abc"));
        let iden2 = Identifier::from((false, "abc2"));
        let iden3 = Identifier::from(&vec![iden, iden2]);

        let name = Name::tuple(&[
            Name::from("A2"),
            Name::from("A1"),
            Name::from("A0"),
        ]);

        let matchings = match_name(&iden3, &name, Position::invisible());
        assert!(matchings.is_err());
    }

    #[test]
    fn range_has_collection_int_as_parent() -> TypeResult<()> {
        let range_name = Name::from(RANGE);
        let int_name = Name::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        let collection_ty = range_name.col_type(&ctx, Position::invisible())?;
        assert_eq!(collection_ty, Some(int_name));
        Ok(())
    }

    #[test]
    fn float_parent_of_int() -> TypeResult<()> {
        let float_name = Name::from(FLOAT);
        let int_name = StringName::from(INT);
        let ctx = Context::default().into_with_primitives().unwrap();

        let clss = ctx.class(&int_name, Position::invisible())?;
        assert!(clss.has_parent(&float_name, &ctx, Position::invisible())?);
        Ok(())
    }

    #[test]
    fn int_not_parent_of_float() -> TypeResult<()> {
        let float_name = StringName::from(FLOAT);
        let int_name = Name::from(INT);
        let ctx = Context::default().into_with_primitives().unwrap();

        let clss = ctx.class(&float_name, Position::invisible())?;
        assert!(!clss.has_parent(&int_name, &ctx, Position::invisible())?);
        Ok(())
    }

    #[test]
    fn slice_not_collection_int_as_parent() {
        let range_name = Name::from(clss::SLICE);

        let ctx = Context::default().into_with_primitives().unwrap();
        let collection_ty = range_name.col_type(&ctx, Position::invisible());
        assert!(collection_ty.is_err());
    }

    #[test]
    fn name_fold() {
        let int_name = Name::from(INT);
        let float_name = Name::from(FLOAT);

        let name1 = Name::from(&HashSet::from([int_name.clone(), float_name.clone()]));
        let name2 = [int_name, float_name].iter().fold(Name::empty(), |name, n| name.union(n));
        assert_eq!(name1, name2);
    }
}
