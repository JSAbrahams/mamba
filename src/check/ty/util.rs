use std::collections::HashMap;

use crate::check::context::name::{Name, NameUnion};
use crate::check::ident::Identifier;
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::Type;
use crate::common::position::Position;

pub fn match_type(
    identifier: &Identifier,
    ty: &NameUnion,
    pos: &Position
) -> TypeResult<HashMap<String, (bool, NameUnion)>> {
    let unions: Vec<HashMap<String, (bool, NameUnion)>> = ty
        .union
        .iter()
        .map(|ty| match_type_direct(identifier, &ty.name, pos))
        .collect::<Result<_, _>>()?;

    let mut final_union: HashMap<String, (bool, Type)> = HashMap::new();
    for union in unions {
        for (id, (mutable, expr_ty)) in union {
            if final_union.contains_key(&id) {
                let (current_mutable, current_expr_ty) =
                    final_union.get(&id).unwrap_or_else(|| unreachable!());
                let new_mutable = mutable && *current_mutable;
                let new_expr_ty = current_expr_ty.clone().union(&expr_ty);
                final_union.insert(id, (new_mutable, new_expr_ty));
            } else {
                final_union.insert(id, (mutable, expr_ty));
            }
        }
    }

    Ok(final_union)
}

pub fn match_type_direct(
    identifier: &Identifier,
    ty: &Name,
    pos: &Position
) -> TypeResult<HashMap<String, (bool, NameUnion)>> {
    match &ty {
        Name::Single { .. } | Name::Fun { .. } =>
            if let Some((mutable, id)) = &identifier.lit {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone(), (*mutable, ty.clone()));
                Ok(mapping)
            } else {
                let msg = format!("Cannot match {} with type {}", identifier, ty);
                Err(vec![TypeErr::new(pos, &msg)])
            },
        Name::Tuple(elements) =>
            if let Some((mutable, id)) = &identifier.lit {
                let mut mapping = HashMap::with_capacity(1);
                mapping.insert(id.clone(), (*mutable, ty.clone()));
                Ok(mapping)
            } else if elements.len() == identifier.fields().len() {
                let sets: Vec<HashMap<_, _>> = identifier
                    .names
                    .iter()
                    .zip(elements)
                    .map(|(identifier, ty)| match_type(&identifier, &ty, pos))
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
