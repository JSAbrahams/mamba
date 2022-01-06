use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::IntoIter;
use std::fmt;
use std::fmt::{Display, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use crate::check::context::{Context, LookupClass};
use crate::check::context::clss::{HasParent, NONE};
use crate::check::ident::Identifier;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod generic;
pub mod python;
mod name_test;

/// A direct name is a string with accompanying generics.
///
/// Useful to denote class and function names, where Tuples and Anonymous
/// functions are not permitted.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectName {
    pub name: String,
    pub generics: Vec<NameUnion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NameVariant {
    Single(DirectName),
    Tuple(Vec<NameUnion>),
    Fun(Vec<NameUnion>, Box<NameUnion>),
}

/// Name is the actual name of a Function, Field, or generic.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    is_nullable: bool,
    is_mutable: bool,
    pub variant: NameVariant,
}

#[derive(Debug, Clone, Eq)]
pub struct NameUnion {
    names: HashSet<Name>,
}

pub trait Union<T> {
    fn union(&self, value: &T) -> Self;
}

pub trait IsSuperSet<T> {
    fn is_superset_of(&self, other: &T, ctx: &Context, pos: &Position) -> TypeResult<bool>;
}

pub trait IsNullable {
    fn is_nullable(&self) -> bool;
}

pub trait AsNullable {
    fn as_nullable(&self) -> Self;
}

pub trait AsMutable {
    fn as_mutable(&self) -> Self;
}

impl AsMutable for NameUnion {
    fn as_mutable(&self) -> Self {
        NameUnion { names: self.names.iter().map(|n| n.as_mutable()).collect() }
    }
}

impl AsMutable for Name {
    fn as_mutable(&self) -> Self { Name { is_mutable: true, ..self.clone() } }
}

impl Union<NameUnion> for NameUnion {
    fn union(&self, name: &NameUnion) -> Self {
        NameUnion { names: self.names.union(&name.names).cloned().collect() }
    }
}

impl Union<DirectName> for NameUnion {
    fn union(&self, name: &DirectName) -> Self {
        let mut names = self.names.clone();
        names.insert(Name::from(name));
        NameUnion { names }
    }
}

impl Union<Name> for NameUnion {
    fn union(&self, name: &Name) -> Self {
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

impl From<&DirectName> for NameUnion {
    fn from(name: &DirectName) -> Self {
        NameUnion { names: HashSet::from_iter(vec![Name::from(name)]) }
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

impl Display for DirectName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let generics = if self.generics.is_empty() {
            String::new()
        } else {
            format!("[{}]", comma_delm(&self.generics))
        };
        write!(f, "{}{}", self.name, generics)
    }
}

impl From<&NameVariant> for Name {
    fn from(variant: &NameVariant) -> Self {
        Name { is_mutable: false, is_nullable: false, variant: variant.clone() }
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mutable = if self.is_mutable { "mut " } else { "" };
        write!(f, "{}{}{}", mutable, self.variant, if self.is_nullable { "?" } else { "" })
    }
}

impl Display for NameVariant {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            NameVariant::Single(direct_name) => write!(f, "{}", direct_name),
            NameVariant::Tuple(names) => write!(f, "({})", comma_delm(names)),
            NameVariant::Fun(args, ret) => write!(f, "({}) -> {}", comma_delm(args), ret)
        }
    }
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

impl From<&Name> for NameUnion {
    fn from(name: &Name) -> Self {
        let names: HashSet<Name> = HashSet::from_iter(vec![name.clone()]);
        NameUnion { names }
    }
}

impl From<&str> for DirectName {
    fn from(name: &str) -> Self { DirectName { name: String::from(name), generics: vec![] } }
}

impl From<&DirectName> for Name {
    fn from(name: &DirectName) -> Self {
        Name {
            is_nullable: false,
            is_mutable: false,
            variant: NameVariant::Single(name.clone()),
        }
    }
}

impl From<&str> for Name {
    fn from(name: &str) -> Self {
        Name {
            is_nullable: false,
            is_mutable: false,
            variant: NameVariant::Single(DirectName::from(name)),
        }
    }
}

impl From<&str> for NameUnion {
    fn from(name: &str) -> Self { NameUnion::from(&Name::from(name)) }
}

impl IsSuperSet<NameVariant> for NameVariant {
    fn is_superset_of(
        &self,
        other: &NameVariant,
        ctx: &Context,
        pos: &Position,
    ) -> TypeResult<bool> {
        match (self, other) {
            (NameVariant::Single(left), NameVariant::Single(right)) =>
                left.is_superset_of(right, ctx, pos),
            (NameVariant::Tuple(left), NameVariant::Tuple(right)) => left
                .iter()
                .map(|l| right.iter().map(move |r| l.is_superset_of(r, ctx, pos)))
                .flatten()
                .collect::<Result<Vec<bool>, _>>()
                .map(|b| b.iter().all(|b| *b)),
            (NameVariant::Fun(left_a, left), NameVariant::Fun(right_a, right)) =>
                Ok(left_a.len() == right_a.len() && left.is_superset_of(right, ctx, pos)? && {
                    let mut all = true;
                    for (left_a, right_a) in left_a.iter().zip(right_a) {
                        all = all && left_a.is_superset_of(right_a, ctx, pos)?;
                    }
                    all
                }),
            _ => Ok(false)
        }
    }
}

impl IsSuperSet<DirectName> for DirectName {
    fn is_superset_of(
        &self,
        other: &DirectName,
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

impl DirectName {
    pub fn new(lit: &str, generics: &[NameUnion]) -> DirectName {
        DirectName { name: String::from(lit), generics: Vec::from(generics) }
    }

    pub fn empty() -> DirectName { DirectName::new("()", &[]) }

    pub fn substitute(
        &self,
        generics: &HashMap<String, Name>,
        pos: &Position,
    ) -> TypeResult<DirectName> {
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
            Ok(DirectName {
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

impl IsNullable for Name {
    fn is_nullable(&self) -> bool { self.is_nullable }
}

impl AsNullable for Name {
    fn as_nullable(&self) -> Self { Name { is_nullable: true, ..self.clone() } }
}

#[allow(clippy::nonminimal_bool)]
impl IsSuperSet<Name> for Name {
    /// Check if name is supertype of other name.
    ///
    /// If self is nullable, then supertype of other if:
    /// - Other is null.
    /// - Or, variant is supertype of other's variant. (Other may or may not be nullable.)
    /// If self is not nullable, then only super type if:
    /// - Other is not nullable.
    /// - And, variant is supertype of other's variant.
    fn is_superset_of(&self, other: &Name, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        if !self.is_empty() && other.is_empty() {
            return Ok(false);
        } else if self.is_nullable() && other.is_null() {
            return Ok(true); // Trivially true
        }

        let nullable_super = self.is_nullable() || (!self.is_nullable() && !other.is_nullable());
        Ok(nullable_super && self.variant.is_superset_of(&other.variant, ctx, pos)?)
    }
}

impl Name {
    pub fn new(lit: &str, generics: &[NameUnion]) -> Name {
        Name::from(&DirectName::new(lit, generics))
    }

    pub fn is_empty(&self) -> bool { self == &Name::empty() }

    pub fn is_null(&self) -> bool {
        match &self.variant {
            NameVariant::Single(DirectName { name, .. }) if name.clone() == String::from(NONE) => true,
            _ => false
        }
    }

    pub fn empty() -> Name { Name::from(&DirectName::empty()) }

    pub fn as_direct(&self, exp: &str, pos: &Position) -> TypeResult<DirectName> {
        match &self.variant {
            NameVariant::Single(name) => Ok(name.clone()),
            other =>
                Err(vec![TypeErr::new(pos, &format!("'{}' is not a valid {} name", other, exp))]),
        }
    }

    pub fn substitute(&self, generics: &HashMap<String, Name>, pos: &Position) -> TypeResult<Name> {
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

        Ok(Name { variant, ..self.clone() })
    }
}

impl IsSuperSet<NameUnion> for NameUnion {
    fn is_superset_of(&self, other: &NameUnion, ctx: &Context, pos: &Position) -> TypeResult<bool> {
        if !self.is_empty() && other.is_empty() {
            return Ok(false);
        }

        for name in &other.names {
            let is_superset = |s_name: &Name| s_name.is_superset_of(&name, ctx, pos);
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
    fn is_nullable(&self) -> bool { self.names.iter().all(|n| n.is_nullable) }
}

impl AsNullable for NameUnion {
    fn as_nullable(&self) -> Self {
        NameUnion { names: self.names.iter().map(|n| n.as_nullable()).collect() }
    }
}

impl NameUnion {
    pub fn new(names: &[Name]) -> NameUnion {
        let names: HashSet<Name> = HashSet::from_iter(Vec::from(names));
        NameUnion { names }
    }

    pub fn is_empty(&self) -> bool { self == &NameUnion::empty() }

    pub fn as_direct(&self, msg: &str, pos: &Position) -> TypeResult<HashSet<DirectName>> {
        self.names.iter().map(|n| n.as_direct(msg, pos)).collect::<Result<_, _>>()
    }

    pub fn empty() -> NameUnion { NameUnion { names: HashSet::new() } }

    pub fn is_null(&self) -> bool {
        self.names.iter().all(|name| name.is_null())
    }

    pub fn names(&self) -> IntoIter<Name> { self.names.clone().into_iter() }

    pub fn substitute(
        &self,
        generics: &HashMap<String, Name>,
        pos: &Position,
    ) -> TypeResult<NameUnion> {
        let names =
            self.names.iter().map(|n| n.substitute(generics, pos)).collect::<Result<_, _>>()?;
        Ok(NameUnion { names })
    }
}

pub fn match_name(
    identifier: &Identifier,
    name: &NameUnion,
    pos: &Position,
) -> TypeResult<HashMap<String, (bool, NameUnion)>> {
    let unions: Vec<HashMap<String, (bool, NameUnion)>> =
        name.names().map(|ty| match_type_direct(identifier, &ty, pos)).collect::<Result<_, _>>()?;

    let mut final_union: HashMap<String, (bool, NameUnion)> = HashMap::new();
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
    name: &Name,
    pos: &Position,
) -> TypeResult<HashMap<String, (bool, NameUnion)>> {
    match &name.variant {
        NameVariant::Single { .. } | NameVariant::Fun { .. } =>
            if let Some((mutable, id)) = &identifier.lit {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone(), (*mutable, NameUnion::from(name)));
                Ok(mapping)
            } else {
                let msg = format!("Cannot match {} with a '{}'", identifier, name);
                Err(vec![TypeErr::new(pos, &msg)])
            },
        NameVariant::Tuple(elements) =>
            if let Some((mutable, id)) = &identifier.lit {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone(), (*mutable, NameUnion::from(name)));
                Ok(mapping)
            } else if elements.len() == identifier.fields().len() {
                let sets: Vec<HashMap<_, _>> = identifier
                    .names
                    .iter()
                    .zip(elements)
                    .map(|(identifier, ty)| match_name(&identifier, &ty, pos))
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
