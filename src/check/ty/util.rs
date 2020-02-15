use crate::check::ident::Identifier;
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::name::actual::ActualTypeName;
use crate::check::ty::name::TypeName;
use crate::common::position::Position;
use std::collections::HashMap;

pub fn match_type(
    identifier: &Identifier,
    type_name: &TypeName,
    pos: &Position
) -> TypeResult<HashMap<String, (bool, TypeName)>> {
    match type_name {
        TypeName::Single { ty } => match &ty.actual {
            ActualTypeName::Single { .. } | ActualTypeName::AnonFun { .. } =>
                if let Some((mutable, id)) = &identifier.lit {
                    let mut mapping = HashMap::with_capacity(1);
                    mapping.insert(id.clone(), (*mutable, type_name.clone()));
                    Ok(mapping)
                } else {
                    let msg = format!("Cannot match {} with type {}", identifier, type_name);
                    Err(vec![TypeErr::new(pos, &msg)])
                },
            ActualTypeName::Tuple { ty_names } =>
                if let Some((mutable, id)) = &identifier.lit {
                    let mut mapping = HashMap::with_capacity(1);
                    mapping.insert(id.clone(), (*mutable, type_name.clone()));
                    Ok(mapping)
                } else if ty_names.len() == identifier.fields().len() {
                    let sets: Vec<HashMap<_, _>> = identifier
                        .names
                        .iter()
                        .zip(ty_names)
                        .map(|(identifier, type_name)| match_type(&identifier, &type_name, pos))
                        .collect::<Result<_, _>>()?;
                    Ok(sets.into_iter().flatten().collect())
                } else {
                    Err(vec![TypeErr::new(
                        pos,
                        &format!(
                            "Cannot iterate over {} with tuple of size {}",
                            ty,
                            identifier.fields().len()
                        )
                    )])
                },
        },
        TypeName::Union { union } => {
            let unions: Vec<HashMap<String, (bool, TypeName)>> = union
                .iter()
                .map(|ty| match_type(identifier, &TypeName::from(ty), pos))
                .collect::<Result<_, _>>()?;
            let mut final_union: HashMap<String, (bool, TypeName)> = HashMap::new();
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
    }
}
