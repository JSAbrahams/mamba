use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use crate::check::checker_result::{TypeErr, TypeResult};
use crate::check::environment::Environment;
use crate::check::ty::actual::ActualType;
use crate::check::ty::ExpressionType;
use crate::check::ty_name::actual::ActualTypeName;
use crate::check::ty_name::TypeName;
use crate::check::util::comma_delimited;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Clone, Debug)]
pub struct Identifier {
    lit:   Option<(bool, String)>,
    names: Vec<Identifier>
}

impl Identifier {
    pub fn fields(&self) -> Vec<(bool, String)> {
        if let Some(lit) = &self.lit {
            vec![lit.clone()]
        } else {
            self.names.iter().map(|name| name.fields()).flatten().collect()
        }
    }

    pub fn as_mutable(&self, mutable: bool) -> Identifier {
        if !mutable {
            return self.clone();
        }

        if let Some((_, id)) = &self.lit {
            Identifier { lit: Some((true, id.clone())), names: self.names.clone() }
        } else {
            Identifier {
                lit:   self.lit.clone(),
                names: self.names.iter().map(|name| name.as_mutable(mutable)).collect()
            }
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some((mutable, lit)) = &self.lit {
            write!(f, "{}{}", if *mutable { "mut " } else { "" }, lit.clone())
        } else {
            write!(f, "({})", comma_delimited(&self.names))
        }
    }
}

impl TryFrom<&AST> for Identifier {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<Identifier> {
        match &ast.node {
            // TODO add mutable field to identifier
            Node::Id { lit } => Ok(Identifier::from(lit.as_str())),
            Node::ExpressionType { expr, mutable, .. } => {
                let identifier = Identifier::try_from(expr.deref())?;
                Ok(identifier.as_mutable(*mutable))
            }
            Node::Tuple { elements } =>
                Ok(Identifier::from(&elements.iter().map(Identifier::try_from).collect::<Result<
                    Vec<Identifier>,
                    _
                >>(
                )?)),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected id or tuple of id's")])
        }
    }
}

impl From<&str> for Identifier {
    fn from(name: &str) -> Self {
        // TODO use mutable field from identifier
        Identifier { lit: Some((false, String::from(name))), names: vec![] }
    }
}

impl From<&Vec<Identifier>> for Identifier {
    fn from(names: &Vec<Identifier>) -> Self { Identifier { lit: None, names: names.to_vec() } }
}

// TODO more elegant map manipulation
#[allow(clippy::map_entry)]
pub fn match_name(
    identifier: &Identifier,
    expr_ty: &ExpressionType,
    env: &Environment,
    pos: &Position
) -> TypeResult<HashMap<String, (bool, ExpressionType)>> {
    match expr_ty {
        ExpressionType::Single { ty } => match &ty.actual_ty() {
            ActualType::Single { .. } | ActualType::AnonFun { .. } =>
                if let Some((mutable, id)) = &identifier.lit {
                    let mut mapping = HashMap::with_capacity(1);
                    mapping.insert(id.clone(), (*mutable, expr_ty.clone()));
                    Ok(mapping)
                } else {
                    let msg = format!("Cannot match {} with type {}", identifier, expr_ty);
                    Err(vec![TypeErr::new(pos, &msg)])
                },
            ActualType::Tuple { types } =>
                if let Some((mutable, id)) = &identifier.lit {
                    let mut mapping = HashMap::with_capacity(1);
                    mapping.insert(id.clone(), (*mutable, expr_ty.clone()));
                    Ok(mapping)
                } else if types.len() == identifier.names.len() {
                    let sets: Vec<HashMap<_, _>> = identifier
                        .names
                        .iter()
                        .zip(types)
                        .map(|(identifier, expr_ty)| match_name(&identifier, expr_ty, env, pos))
                        .collect::<Result<_, _>>()?;
                    Ok(sets.into_iter().flatten().collect())
                } else {
                    Err(vec![TypeErr::new(
                        pos,
                        &format!(
                            "Cannot iterate over {} with tuple of size {}",
                            ty,
                            identifier.names.len()
                        )
                    )])
                },
        },
        ExpressionType::Union { union } => {
            let unions: Vec<HashMap<String, (bool, ExpressionType)>> = union
                .iter()
                .map(|ty| match_name(identifier, &ExpressionType::from(ty), env, pos))
                .collect::<Result<_, _>>()?;
            let mut final_union: HashMap<String, (bool, ExpressionType)> = HashMap::new();
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
                } else if ty_names.len() == identifier.names.len() {
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
                            identifier.names.len()
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
