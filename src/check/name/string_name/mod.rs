use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use crate::check::context::{Context, function, LookupClass};
use crate::check::context::clss::{ANY, GetFun, HasParent};
use crate::check::context::function::union::FunUnion;
use crate::check::name::{ColType, Empty, IsSuperSet, Substitute, TEMP, Union};
use crate::check::name::Name;
use crate::check::name::name_variant::NameVariant;
use crate::check::name::true_name::{IsTemp, TrueName};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod generic;

/// Useful to denote class and function names, where Tuples and Anonymous functions are not permitted.
#[derive(Debug, Clone, Eq)]
pub struct StringName {
    pub name: String,
    pub generics: Vec<Name>,
}

impl PartialEq for StringName {
    /// Tuple and Callable are converted to their full names internally before checking equality.
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && {
            // Force conversion
            let s_generics: Vec<Name> = self.generics.iter().map(Name::full_name).collect();
            let o_generics: Vec<Name> = other.generics.iter().map(Name::full_name).collect();

            s_generics == o_generics
        }
    }
}

impl Name {
    fn full_name(&self) -> Self {
        Name { names: self.names.iter().map(TrueName::full_name).collect(), ..self.clone() }
    }
}

impl TrueName {
    fn full_name(&self) -> Self {
        TrueName { variant: self.variant.full_name(), ..self.clone() }
    }
}

impl NameVariant {
    fn full_name(&self) -> Self {
        NameVariant::Single(StringName::from(self))
    }
}

impl Hash for StringName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.generics.iter().map(Name::full_name).for_each(|n| n.hash(state));
    }
}

impl PartialOrd<Self> for StringName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp = self.name.partial_cmp(&other.name);
        if let Some(Ordering::Equal) = cmp {
            self.generics.partial_cmp(&other.generics)
        } else {
            cmp
        }
    }
}

impl Ord for StringName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl Display for StringName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let generics = if self.generics.is_empty() {
            String::new()
        } else {
            format!("[{}]", comma_delm(&self.generics))
        };
        write!(f, "{}{}", self.name, generics)
    }
}

impl ColType for StringName {
    /// Checks if type has iterator by checking __iter__().
    /// If so, check the return type of the __next__() method and return that.
    fn col_type(&self, ctx: &Context, pos: Position) -> TypeResult<Option<Name>> {
        if let Ok(clss) = ctx.class(self, pos) {
            let fun_name = StringName::from(function::python::ITER);
            if let Ok(fun) = clss.fun(&fun_name, ctx, pos) {
                let iter_name = fun.ret_ty;
                if let Ok(iter_class) = ctx.class(&iter_name, pos) {
                    let next_name = StringName::from(function::python::NEXT);
                    let fun: FunUnion = iter_class.fun(&next_name, ctx, pos)?;
                    let ret_name =
                        fun.union.iter().fold(Name::empty(), |name, i| name.union(&i.ret_ty));
                    Ok(Some(ret_name))
                } else {
                    let msg = format!("Cannot find iterator '{iter_name}' for iterable type '{self}'");
                    Err(vec![TypeErr::new(pos, &msg)])
                }
            } else {
                let msg = format!("Type '{self}' is not iterable, it does not define an iterator.");
                Err(vec![TypeErr::new(pos, &msg)])
            }
        } else {
            Err(vec![TypeErr::new(pos, &format!("'{self}' is undefined"))])
        }
    }
}

impl From<&str> for StringName {
    fn from(name: &str) -> Self {
        StringName { name: String::from(name), generics: vec![] }
    }
}

impl IsSuperSet<StringName> for StringName {
    fn is_superset_of(&self, other: &StringName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        Ok(self.name == ANY ||
            ctx.class(other, pos)?.has_parent(self, ctx, pos)?
                && self
                .generics
                .iter()
                .flat_map(|n| other.generics.iter().map(move |o| n.is_superset_of(o, ctx, pos)))
                .collect::<Result<Vec<bool>, _>>()?
                .iter()
                .all(|b| *b))
    }
}

impl From<&StringName> for Name {
    fn from(name: &StringName) -> Self {
        Name { names: HashSet::from_iter(vec![TrueName::from(name)]), is_interchangeable: false }
    }
}

impl Union<StringName> for Name {
    fn union(&self, name: &StringName) -> Self {
        let mut names = self.names.clone();
        names.insert(TrueName::from(name));
        Name { names, ..self.clone() }
    }
}

impl Empty for StringName {
    fn is_empty(&self) -> bool {
        self == &StringName::empty()
    }

    fn empty() -> StringName {
        StringName::new("()", &[])
    }
}

impl Substitute for StringName {
    fn substitute(&self, generics: &HashMap<Name, Name>, pos: Position) -> TypeResult<StringName> {
        if let Some(name) = generics.get(&Name::from(self)) {
            let string_names = name.as_direct();
            if string_names.len() > 1 {
                let msg = format!("Cannot substitute type union {name}");
                return Err(vec![TypeErr::new(pos, &msg)]);
            }

            if let Some(string_name) = string_names.iter().next() {
                return Ok(string_name.clone());
            }

            Err(vec![TypeErr::new(pos, &format!("{name} incorrect name"))])
        } else {
            Ok(StringName {
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

impl IsTemp for StringName {
    fn is_temp(&self) -> bool {
        self.name.starts_with(TEMP)
    }
}

impl StringName {
    pub fn new(lit: &str, generics: &[Name]) -> StringName {
        StringName { name: String::from(lit), generics: Vec::from(generics) }
    }
}

#[cfg(test)]
mod test {
    use crate::check::context::clss::{BOOL, HasParent, INT, STRING};
    use crate::check::context::LookupClass;
    use crate::check::name::IsSuperSet;
    use crate::check::name::string_name::StringName;
    use crate::common::position::Position;
    use crate::Context;

    #[test]
    fn bool_not_super_of_int() {
        let (name_1, name_2) = (StringName::from(BOOL), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn int_not_parent_of_bool() {
        let (name_1, name_2) = (StringName::from(BOOL), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();

        let bool_class = ctx.class(&name_1, Position::default()).expect("bool class");
        assert!(!bool_class.has_parent(&name_2, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn bool_not_parent_of_int() {
        let (name_1, name_2) = (StringName::from(BOOL), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();

        let int_class = ctx.class(&name_2, Position::default()).expect("int class");
        assert!(!int_class.has_parent(&name_1, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn string_parent_of_string() {
        let (name_1, name_2) = (StringName::from(STRING), StringName::from(STRING));
        let ctx = Context::default().into_with_primitives().unwrap();

        let string_class = ctx.class(&name_2, Position::default()).expect("int class");
        assert!(string_class.has_parent(&name_1, &ctx, Position::default()).unwrap())
    }

    #[test]
    fn string_not_super_of_int() {
        let (name_1, name_2) = (StringName::from(STRING), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::default()).unwrap())
    }
}
