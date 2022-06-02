use std::cmp::Ordering;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

use crate::check::context::Context;
use crate::check::name::{ColType, IsSuperSet};
use crate::check::name::Name;
use crate::check::name::stringname::StringName;
use crate::check::result::TypeResult;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NameVariant {
    Single(StringName),
    Tuple(Vec<Name>),
    Fun(Vec<Name>, Box<Name>),
}

impl PartialOrd<Self> for NameVariant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (NameVariant::Single(l_name), NameVariant::Single(r_name)) => {
                l_name.partial_cmp(r_name)
            }
            (NameVariant::Tuple(l_name), NameVariant::Tuple(r_name)) => l_name.partial_cmp(r_name),
            (NameVariant::Fun(l_args, l_ret), NameVariant::Fun(r_args, r_ret)) => {
                let cmp = l_args.partial_cmp(r_args);
                if let Some(Ordering::Equal) = cmp {
                    l_ret.partial_cmp(r_ret)
                } else {
                    cmp
                }
            }
            _ => todo!(),
        }
    }
}

impl Ord for NameVariant {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Display for NameVariant {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            NameVariant::Single(direct_name) => write!(f, "{}", direct_name),
            NameVariant::Tuple(names) => write!(f, "({})", comma_delm(names)),
            NameVariant::Fun(args, ret) => write!(f, "({}) -> {}", comma_delm(args), ret),
        }
    }
}

impl ColType for NameVariant {
    fn col_type(&self, ctx: &Context, pos: Position) -> TypeResult<Option<Name>> {
        if let NameVariant::Single(string_name) = self {
            string_name.col_type(ctx, pos)
        } else {
            Ok(None)
        }
    }
}

impl IsSuperSet<NameVariant> for NameVariant {
    fn is_superset_of(
        &self,
        other: &NameVariant,
        ctx: &Context,
        pos: Position,
    ) -> TypeResult<bool> {
        match (self, other) {
            (NameVariant::Single(left), NameVariant::Single(right)) => {
                left.is_superset_of(right, ctx, pos)
            }
            (NameVariant::Tuple(left), NameVariant::Tuple(right)) => left
                .iter()
                .flat_map(|l| right.iter().map(move |r| l.is_superset_of(r, ctx, pos)))
                .collect::<Result<Vec<bool>, _>>()
                .map(|b| b.iter().all(|b| *b)),
            (NameVariant::Fun(left_a, left), NameVariant::Fun(right_a, right)) => {
                Ok(left_a.len() == right_a.len() && left.is_superset_of(right, ctx, pos)? && {
                    let mut all = true;
                    for (left_a, right_a) in left_a.iter().zip(right_a) {
                        all = all && left_a.is_superset_of(right_a, ctx, pos)?;
                    }
                    all
                })
            }
            _ => Ok(false),
        }
    }
}
