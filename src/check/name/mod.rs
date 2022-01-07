use std::collections::HashMap;

use crate::check::context::Context;
use crate::check::ident::Identifier;
use crate::check::name::nameunion::NameUnion;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

pub mod stringname;
pub mod truename;
pub mod nameunion;
pub mod namevariant;

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

pub fn match_name(identifier: &Identifier, name: &NameUnion, pos: &Position) -> TypeResult<HashMap<String, (bool, NameUnion)>> {
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

pub fn match_type_direct(identifier: &Identifier, name: &TrueName, pos: &Position) -> TypeResult<HashMap<String, (bool, NameUnion)>> {
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
