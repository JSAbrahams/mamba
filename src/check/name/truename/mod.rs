use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

use crate::check::context::clss::NONE;
use crate::check::context::Context;
use crate::check::name::{AsMutable, AsNullable, ColType, IsNullable, IsSuperSet};
use crate::check::name::Name;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

pub mod generic;
pub mod python;

/// Name is the actual truename of a Function, Field, or generic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrueName {
    is_nullable: bool,
    is_mutable: bool,
    pub variant: NameVariant,
}

impl AsMutable for TrueName {
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
        let mutable = if self.is_mutable { "mut " } else { "" };
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

impl IsNullable for TrueName {
    fn is_nullable(&self) -> bool { self.is_nullable }
}

impl AsNullable for TrueName {
    fn as_nullable(&self) -> Self { TrueName { is_nullable: true, ..self.clone() } }
}

impl PartialEq<StringName> for TrueName {
    fn eq(&self, other: &StringName) -> bool {
        match &self.variant {
            NameVariant::Single(string_name) => string_name == other,
            _ => false
        }
    }
}

#[allow(clippy::nonminimal_bool)]
impl IsSuperSet<TrueName> for TrueName {
    /// Check if name is supertype of other name.
    ///
    /// If self is nullable, then supertype of other if:
    /// - Other is null.
    /// - Or, variant is supertype of other's variant. (Other may or may not be nullable.)
    /// If self is not nullable, then only super type if:
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

impl TrueName {
    pub fn new(lit: &str, generics: &[Name]) -> TrueName {
        TrueName::from(&StringName::new(lit, generics))
    }

    pub fn is_empty(&self) -> bool { self == &TrueName::empty() }

    pub fn is_null(&self) -> bool {
        matches!(&self.variant, NameVariant::Single(StringName { name, .. }) if name.clone() == *NONE)
    }

    pub fn empty() -> TrueName { TrueName::from(&StringName::empty()) }

    pub fn as_direct(&self, exp: &str, pos: Position) -> TypeResult<StringName> {
        match &self.variant {
            NameVariant::Single(name) => Ok(name.clone()),
            other =>
                Err(vec![TypeErr::new(pos, &format!("'{}' is not a valid {} name", other, exp))]),
        }
    }

    pub fn substitute(&self, generics: &HashMap<Name, Name>, pos: Position) -> TypeResult<TrueName> {
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
