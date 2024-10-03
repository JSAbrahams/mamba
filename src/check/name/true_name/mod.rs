use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;
use std::iter::FromIterator;

use crate::check::context::clss::NONE;
use crate::check::context::{clss, Context};
use crate::check::name::string_name::StringName;
use crate::check::name::{
    Any, ColType, ContainsTemp, Empty, IsSuperSet, Mutable, Name, Nullable, Substitute,
    TupleCallable, Union,
};
use crate::check::result::TypeResult;
use crate::common::position::Position;

pub mod generic;
pub mod python;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrueName {
    pub is_nullable: bool,
    pub is_mutable: bool,
    pub variant: StringName,
}

pub trait IsTemp {
    fn is_temp(&self) -> bool;
}

pub trait MatchTempName {
    fn temp_map(
        &self,
        other: &StringName,
        mapping: HashMap<Name, Name>,
        pos: Position,
    ) -> TypeResult<HashMap<Name, Name>>;
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

impl Any for TrueName {
    fn any() -> Self {
        TrueName::from(clss::ANY)
    }
}

impl Ord for TrueName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Mutable for TrueName {
    fn as_mutable(&self) -> Self {
        TrueName {
            is_mutable: true,
            ..self.clone()
        }
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
        write!(
            f,
            "{}{}{}",
            mutable,
            self.variant,
            if self.is_nullable { "?" } else { "" }
        )
    }
}

impl From<&StringName> for TrueName {
    fn from(name: &StringName) -> Self {
        TrueName {
            is_nullable: false,
            is_mutable: true,
            variant: name.clone(),
        }
    }
}

impl From<&str> for TrueName {
    fn from(name: &str) -> Self {
        TrueName::from(&StringName::from(name))
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
        Name {
            names,
            ..self.clone()
        }
    }
}

impl Empty for TrueName {
    fn is_empty(&self) -> bool {
        self.variant.is_empty()
    }
    fn empty() -> TrueName {
        TrueName::from(&StringName::empty())
    }
}

impl Nullable for TrueName {
    fn is_nullable(&self) -> bool {
        self.is_nullable
    }
    fn is_null(&self) -> bool {
        self.variant.name == NONE
    }
    fn as_nullable(&self) -> Self {
        TrueName {
            is_nullable: true,
            ..self.clone()
        }
    }
}

impl Substitute for TrueName {
    fn substitute(&self, generics: &HashMap<Name, Name>, pos: Position) -> TypeResult<TrueName> {
        Ok(TrueName {
            variant: self.variant.substitute(generics, pos)?,
            ..self.clone()
        })
    }
}

impl From<&Vec<TrueName>> for Name {
    fn from(names: &Vec<TrueName>) -> Self {
        let names: HashSet<TrueName> = HashSet::from_iter(names.iter().cloned());
        Name {
            names,
            is_interchangeable: false,
        }
    }
}

impl IsTemp for TrueName {
    fn is_temp(&self) -> bool {
        self.variant.is_temp()
    }
}

impl ContainsTemp for TrueName {
    fn contains_temp(&self) -> bool {
        self.variant.contains_temp()
    }
}

impl MatchTempName for TrueName {
    fn temp_map(
        &self,
        other: &StringName,
        mapping: HashMap<Name, Name>,
        pos: Position,
    ) -> TypeResult<HashMap<Name, Name>> {
        self.variant.temp_map(other, mapping, pos)
    }
}

impl TupleCallable<bool, Vec<Name>, Name> for TrueName {
    fn tuple(names: &[Name]) -> Self {
        Self {
            is_nullable: false,
            is_mutable: true,
            variant: StringName::tuple(names),
        }
    }

    fn callable(args: &[Name], ret_ty: &Name) -> Self {
        Self {
            is_nullable: false,
            is_mutable: true,
            variant: StringName::callable(args, ret_ty),
        }
    }

    fn is_tuple(&self) -> bool {
        self.variant.is_tuple()
    }

    fn is_callable(&self) -> bool {
        self.variant.is_callable()
    }

    fn elements(&self, pos: Position) -> TypeResult<Vec<Name>> {
        self.variant.elements(pos)
    }

    fn args(&self, pos: Position) -> TypeResult<Vec<Name>> {
        self.variant.args(pos)
    }

    fn ret_ty(&self, pos: Position) -> TypeResult<Name> {
        self.variant.ret_ty(pos)
    }
}

impl TrueName {
    pub fn new(lit: &str, generics: &[Name]) -> TrueName {
        TrueName::from(&StringName::new(lit, generics))
    }

    pub fn trim(&self, ty: &str) -> Option<TrueName> {
        self.variant.trim(ty).map(|variant| TrueName {
            variant,
            ..self.clone()
        })
    }
}

#[cfg(test)]
mod test {
    use crate::check::context::clss::{BOOL, COMPLEX, INT, STRING};
    use crate::check::name::true_name::TrueName;
    use crate::check::name::{IsSuperSet, Nullable};
    use crate::common::position::Position;
    use crate::Context;

    #[test]
    fn nullable_is_super() {
        let name_1 = TrueName::from("Str").as_nullable();
        let name_2 = TrueName::from("Str");

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(name_1
            .is_superset_of(&name_2, &ctx, Position::invisible())
            .unwrap())
    }

    #[test]
    fn bool_not_super_of_int() {
        let name_1 = TrueName::from(BOOL);
        let name_2 = TrueName::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1
            .is_superset_of(&name_2, &ctx, Position::invisible())
            .unwrap())
    }

    #[test]
    fn string_not_super_of_int() {
        let name_1 = TrueName::from(STRING);
        let name_2 = TrueName::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1
            .is_superset_of(&name_2, &ctx, Position::invisible())
            .unwrap())
    }

    #[test]
    fn complex_super_of_int() {
        let name_1 = TrueName::from(COMPLEX);
        let name_2 = TrueName::from(INT);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(name_1
            .is_superset_of(&name_2, &ctx, Position::invisible())
            .unwrap())
    }
}
