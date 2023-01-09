use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::hash::{Hash, Hasher};

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::check::context::{clss, Context};
use crate::check::name::{ColType, ContainsTemp, Empty, IsSuperSet};
use crate::check::name::Name;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::{IsTemp, TempMap, TrueName};
use crate::check::result::{TypeErr, TypeResult};
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
            NameVariant::Single(direct_name) => write!(f, "{direct_name}"),
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

impl IsTemp for NameVariant {
    fn is_temp(&self) -> bool {
        if let NameVariant::Single(string_name) = self {
            string_name.is_temp()
        } else {
            false
        }
    }
}

impl ContainsTemp for NameVariant {
    fn contains_temp(&self) -> bool {
        match &self {
            NameVariant::Single(string_name) =>
                string_name.is_temp() || string_name.generics.iter().any(Name::contains_temp),
            NameVariant::Tuple(elements) => elements.iter().any(Name::contains_temp),
            NameVariant::Fun(args, ret) =>
                args.iter().any(Name::contains_temp) || ret.contains_temp()
        }
    }
}

impl TempMap for NameVariant {
    fn temp_map(&self, other: &NameVariant, mut mapping: HashMap<Name, Name>, pos: Position)
                -> TypeResult<HashMap<Name, Name>> {
        macro_rules! map_names {
            ($l_names:expr, $r_names:expr) => {{
                for unified_gen in $l_names.iter().zip_longest($r_names) {
                    match unified_gen {
                        Both(left, right) => {
                            mapping = left.temp_map_with_mapping(right, mapping, pos)?;
                        }
                        Left(unbound) => {
                            let msg = format!("while unifying {self} and {other}, no generic for {unbound}");
                            return Err(vec![TypeErr::new(pos, &msg)])
                        }
                        Right(unbound) => {
                            let msg = format!("while unifying {self} and {other}, no generic for {unbound}");
                            return Err(vec![TypeErr::new(pos, &msg)])
                        }
                    }
                }

                mapping
            }}
        }

        match (&self, other) {
            (NameVariant::Single(s_name), NameVariant::Single(o_name)) => {
                if s_name.is_temp() {
                    mapping.insert(Name::from(s_name.name.as_str()), Name::from(other));
                }
                Ok(map_names!(&s_name.generics, &o_name.generics))
            }
            (NameVariant::Tuple(s_elements), NameVariant::Tuple(o_elements)) =>
                Ok(map_names!(s_elements, o_elements)),
            (NameVariant::Fun(s_args, s_ret), NameVariant::Fun(o_args, o_ret)) => {
                let mapping = map_names!(s_args, o_args);
                s_ret.temp_map_with_mapping(o_ret, mapping, pos)
            }

            (NameVariant::Single(_), _) => {
                // If self is StringName, just insert directly
                mapping.insert(Name::from(self), Name::from(other));
                Ok(mapping)
            }
            (NameVariant::Tuple(elements), _) =>
                elements.iter().fold(Ok(mapping), |acc, n| if let Ok(acc) = acc {
                    n.temp_map_with_mapping(&Name::from(other), acc, pos)
                } else {
                    acc
                }),
            (NameVariant::Fun(args, ret), _) => {
                let mapping = args.iter().fold(Ok(mapping), |acc, n| if let Ok(acc) = acc {
                    n.temp_map_with_mapping(&Name::from(other), acc, pos)
                } else {
                    acc
                })?;
                ret.temp_map_with_mapping(&Name::from(other), mapping, pos)
            }
        }
    }
}

impl NameVariant {
    pub fn trim(&self, ty: &str) -> Option<Self> {
        match &self {
            NameVariant::Single(old) => {
                if old.name == ty {
                    None
                } else {
                    let generics: Vec<Name> = old.generics.iter().map(|n| n.trim(ty)).filter(|n| !n.is_empty()).collect();
                    let new = StringName::new(&old.name, generics.as_slice());
                    Some(NameVariant::Single(new))
                }
            }
            NameVariant::Tuple(elements) => {
                let elements: Vec<Name> = elements.iter().map(|n| n.trim(ty)).filter(|n| !n.is_empty()).collect();
                if elements.is_empty() {
                    None
                } else {
                    Some(NameVariant::Tuple(elements))
                }
            }
            NameVariant::Fun(args, ret) => {
                let args: Vec<Name> = args.iter().map(|n| n.trim(ty)).filter(|n| !n.is_empty()).collect();
                let ret = ret.trim(ty);
                if args.is_empty() || ty.is_empty() {
                    None
                } else {
                    Some(NameVariant::Fun(args, Box::from(ret)))
                }
            }
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
