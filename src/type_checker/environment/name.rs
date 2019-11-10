use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::util::comma_delimited;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Identifier {
    lit:   Option<(bool, String)>,
    names: Vec<Identifier>
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
            Node::IdType { id, .. } => Identifier::try_from(id.deref()),
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
