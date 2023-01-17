use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;
use std::iter::FromIterator;

use itertools::EitherOrBoth::Both;
use itertools::Itertools;

use crate::check::context::{Context, function, LookupClass};
use crate::check::context::clss::{CALLABLE, GetFun, HasParent, TUPLE, UNION};
use crate::check::name::{ColType, ContainsTemp, Empty, IsSuperSet, NameMap, Substitute, TEMP, TupleCallable, Union};
use crate::check::name::Name;
use crate::check::name::true_name::{IsTemp, MatchTempName, TrueName};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod generic;

/// Useful to denote class and function names, where Tuples and Anonymous functions are not permitted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StringName {
    pub name: String,
    pub generics: Vec<Name>,
}

impl Display for StringName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}{}", self.name, if self.generics.is_empty() {
            String::new()
        } else {
            format!("[{}]", comma_delm(&self.generics))
        })
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

                    let ret_name = iter_class.iter()
                        .map(|c| c.fun(&next_name, ctx, pos))
                        .map(|f| f.map(|f| f.ret_ty))
                        .collect::<TypeResult<Vec<Name>>>()?
                        .iter()
                        .fold(Name::empty(), |acc, n| acc.union(n));

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

impl From<&TrueName> for StringName {
    fn from(value: &TrueName) -> Self {
        value.variant.clone()
    }
}

impl From<&str> for StringName {
    fn from(name: &str) -> Self {
        StringName { name: String::from(name), generics: vec![] }
    }
}

impl IsSuperSet<StringName> for StringName {
    fn is_superset_of(&self, other: &StringName, ctx: &Context, pos: Position) -> TypeResult<bool> {
        ctx.class(other, pos)?.has_parent(self, ctx, pos)
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
        self == &StringName::empty() || (self.name == TUPLE && self.generics.is_empty())
    }

    fn empty() -> StringName {
        StringName::new("()", &[])
    }
}

impl Substitute for StringName {
    fn substitute(&self, generics: &HashMap<Name, Name>, pos: Position) -> TypeResult<StringName> {
        if let Some(name) = generics.get(&Name::from(self)) {
            let string_names = name.as_direct();
            if string_names.is_empty() {
                let msg = format!("Cannot substitute type union {name}");
                Err(vec![TypeErr::new(pos, &msg)])
            } else if string_names.len() == 1 {
                Ok(string_names.iter().next().expect("Unreachable").clone())
            } else {
                let names: Vec<Name> = string_names.iter().map(Name::from).collect();
                Ok(StringName::new(UNION, names.as_slice()))
            }
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

impl ContainsTemp for StringName {
    fn contains_temp(&self) -> bool {
        self.is_temp() || self.generics.iter().clone().any(|n| n.contains_temp())
    }
}

impl TupleCallable<bool, Vec<Name>, Name> for StringName {
    fn tuple(names: &[Name]) -> Self {
        StringName::new(TUPLE, names)
    }

    fn callable(args: &[Name], ret_ty: &Name) -> Self {
        let args = Name::from(&StringName::new("", args));
        StringName::new(CALLABLE, &[args, ret_ty.clone()])
    }

    fn is_tuple(&self) -> bool {
        self.name == TUPLE
    }

    fn is_callable(&self) -> bool {
        self.name == CALLABLE
    }

    fn elements(&self, pos: Position) -> TypeResult<Vec<Name>> {
        if self.name == TUPLE {
            Ok(self.generics.clone())
        } else {
            Err(vec![TypeErr::new(pos, &format!("{self} is not a tuple"))])
        }
    }

    fn args(&self, pos: Position) -> TypeResult<Vec<Name>> {
        if self.name == CALLABLE {
            if self.generics.len() == 2 {
                let args = self.generics.get(0).expect("Unreachable");
                if let Some(first) = args.names.iter().next() {
                    Ok(first.variant.generics.clone())
                } else {
                    panic!("Malformed callable args: {}", self);
                }
            } else {
                Err(vec![TypeErr::new(pos, &format!("{self} is not a malformed callable"))])
            }
        } else {
            Err(vec![TypeErr::new(pos, &format!("{self} is not a callable"))])
        }
    }

    fn ret_ty(&self, pos: Position) -> TypeResult<Name> {
        if self.name == CALLABLE {
            if self.generics.len() == 2 {
                Ok(self.generics.get(1).expect("Unreachable").clone())
            } else {
                Err(vec![TypeErr::new(pos, &format!("{self} is not a malformed callable"))])
            }
        } else {
            Err(vec![TypeErr::new(pos, &format!("{self} is not a callable"))])
        }
    }
}

impl MatchTempName for StringName {
    fn temp_map(&self, other: &StringName, mut mapping: NameMap, pos: Position) -> TypeResult<NameMap> {
        if self.name.starts_with(TEMP) {
            mapping.insert(Name::from(self.name.as_str()), Name::from(other));
        } else if self.name != other.name {
            return Err(vec![TypeErr::new(pos, &format!("Cannot unify {self} and {other}"))]);
        }

        for either in self.generics.iter().zip_longest(&other.generics) {
            match either {
                Both(self_generic, other_generic) => {
                    mapping = self_generic.temp_map_with_mapping(other_generic, mapping, pos)?;
                }
                _ => {
                    return Err(vec![TypeErr::new(pos, &format!("Cannot unify {self} and {other}"))]);
                }
            }
        }

        Ok(mapping)
    }
}

impl StringName {
    pub fn new(lit: &str, generics: &[Name]) -> StringName {
        StringName { name: String::from(lit), generics: Vec::from(generics) }
    }

    pub fn trim(&self, ty: &str) -> Option<Self> {
        if self.name == ty {
            None
        } else {
            let generics: Vec<Name> = self.generics.iter().map(|n| n.trim(ty)).filter(|n| !n.is_empty()).collect();
            Some(StringName::new(&self.name, generics.as_slice()))
        }
    }

    pub fn match_name(&self, other: &StringName, pos: Position) -> TypeResult<NameMap> {
        let mut mapping = HashMap::new();
        self.match_name_helper(other, &mut mapping, pos)?;
        Ok(mapping)
    }

    pub(crate) fn match_name_helper(&self, other: &StringName, mapping: &mut NameMap, pos: Position) -> TypeResult<()> {
        mapping.insert(Name::from(self.name.as_str()), Name::from(other.name.as_str()));
        for either in self.generics.iter().zip_longest(&other.generics) {
            match either {
                Both(self_generic, other_generic) => {
                    self_generic.match_name_helper(other_generic, mapping, pos)?;
                }
                _ => {
                    return Err(vec![TypeErr::new(pos, &format!("Cannot unify {self} and {other}"))]);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::check::context::clss::{ANY, BOOL, HasParent, INT, STRING};
    use crate::check::context::LookupClass;
    use crate::check::name::IsSuperSet;
    use crate::check::name::string_name::StringName;
    use crate::common::position::Position;
    use crate::Context;

    #[test]
    fn any_super_of_int() {
        let (name_1, name_2) = (StringName::from(ANY), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(name_1.is_superset_of(&name_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn bool_not_super_of_int() {
        let (name_1, name_2) = (StringName::from(BOOL), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn int_not_parent_of_bool() {
        let (name_1, name_2) = (StringName::from(BOOL), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();

        let bool_class = ctx.class(&name_1, Position::invisible()).expect("bool class");
        assert!(!bool_class.has_parent(&name_2, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn bool_not_parent_of_int() {
        let (name_1, name_2) = (StringName::from(BOOL), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();

        let int_class = ctx.class(&name_2, Position::invisible()).expect("int class");
        assert!(!int_class.has_parent(&name_1, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn string_parent_of_string() {
        let (name_1, name_2) = (StringName::from(STRING), StringName::from(STRING));
        let ctx = Context::default().into_with_primitives().unwrap();

        let string_class = ctx.class(&name_2, Position::invisible()).expect("int class");
        assert!(string_class.has_parent(&name_1, &ctx, Position::invisible()).unwrap())
    }

    #[test]
    fn string_not_super_of_int() {
        let (name_1, name_2) = (StringName::from(STRING), StringName::from(INT));
        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!name_1.is_superset_of(&name_2, &ctx, Position::invisible()).unwrap())
    }
}
