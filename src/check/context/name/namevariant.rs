use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

use crate::check::context::Context;
use crate::check::context::name::IsSuperSet;
use crate::check::context::name::NameUnion;
use crate::check::context::name::stringname::StringName;
use crate::check::result::TypeResult;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NameVariant {
    Single(StringName),
    Tuple(Vec<NameUnion>),
    Fun(Vec<NameUnion>, Box<NameUnion>),
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