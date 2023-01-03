use std::cmp::Ordering;
use std::fmt::{Display, Error, Formatter};
use std::hash::{Hash, Hasher};

use crate::check::context::{clss, Context};
use crate::check::name::{ColType, IsSuperSet};
use crate::check::name::Name;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::{IsUndefined, TrueName};
use crate::check::result::TypeResult;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

#[derive(Debug, Clone, Eq)]
pub enum NameVariant {
    Single(StringName),
    Tuple(Vec<Name>),
    Fun(Vec<Name>, Box<Name>),
}

impl PartialEq for NameVariant {
    fn eq(&self, other: &Self) -> bool {
        StringName::from(self) == StringName::from(other)
    }
}

impl Hash for NameVariant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        StringName::from(self).hash(state);
    }
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
            (NameVariant::Single(_), _) => Some(Ordering::Less),

            (NameVariant::Tuple(_), NameVariant::Single(_)) => Some(Ordering::Greater),
            (NameVariant::Tuple(_), _) => Some(Ordering::Less),

            (NameVariant::Fun(..), _) => Some(Ordering::Greater)
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
            (NameVariant::Single(left), right) => match right {
                NameVariant::Tuple(..) if left.name == clss::TUPLE => Ok(true), // ignore generics
                NameVariant::Fun(..) if left.name == clss::CALLABLE => Ok(true), // ignore generics
                _ => Ok(false)
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

impl From<&NameVariant> for StringName {
    fn from(name_variant: &NameVariant) -> Self {
        match name_variant {
            NameVariant::Single(name) => name.clone(),
            NameVariant::Tuple(names) => StringName::new(clss::TUPLE, names),
            NameVariant::Fun(args, ret) => {
                let args = Name::from(&NameVariant::Tuple(args.clone()));
                StringName::new(clss::CALLABLE, &[args, *ret.clone()])
            }
        }
    }
}

impl From<&NameVariant> for Name {
    fn from(name: &NameVariant) -> Self {
        Name::from(&vec![TrueName::from(name)])
    }
}

impl IsUndefined for NameVariant {
    fn is_undefined(&self) -> bool {
        if let NameVariant::Single(string_name) = self {
            string_name.is_undefined()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::check::context::clss::{BOOL, INT, STRING};
    use crate::check::name::IsSuperSet;
    use crate::check::name::name_variant::NameVariant;
    use crate::check::name::string_name::StringName;
    use crate::common::position::Position;
    use crate::Context;

    #[test]
    fn bool_not_super_of_int() {
        let name_1 = NameVariant::Single(StringName::from(BOOL));
        let name_2 = NameVariant::Single(StringName::from(INT));

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn string_not_super_of_int() {
        let name_1 = NameVariant::Single(StringName::from(STRING));
        let name_2 = NameVariant::Single(StringName::from(INT));

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }
}
