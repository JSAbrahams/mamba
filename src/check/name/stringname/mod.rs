use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;

use crate::check::context::{Context, function, LookupClass};
use crate::check::context::clss::HasParent;
use crate::check::name::{ColType, IsSuperSet};
use crate::check::name::{Name, Union};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub mod generic;

/// A direct truename is a string with accompanying generics.
///
/// Useful to denote class and function names, where Tuples and Anonymous
/// functions are not permitted.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringName {
    pub name: String,
    pub generics: Vec<Name>,
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
                    let fun = iter_class.fun(&next_name, ctx, pos)?;
                    let ret_name =
                        fun.union.iter().fold(Name::empty(), |name, i| name.union(&i.ret_ty));
                    Ok(Some(ret_name))
                } else {
                    let msg = format!(
                        "Cannot find iterator '{}' for iterable type '{}'",
                        iter_name, self
                    );
                    Err(vec![TypeErr::new(pos, &msg)])
                }
            } else {
                let msg =
                    format!("Type '{}' is not iterable, it does not define an iterator.", self);
                Err(vec![TypeErr::new(pos, &msg)])
            }
        } else {
            let msg = format!("'{}' is undefined", self);
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

impl From<&str> for StringName {
    fn from(name: &str) -> Self {
        StringName { name: String::from(name), generics: vec![] }
    }
}

impl IsSuperSet<StringName> for StringName {
    fn is_superset_of(
        &self,
        other: &StringName,
        ctx: &Context,
        pos: Position,
    ) -> TypeResult<bool> {
        Ok(ctx.class(other, pos)?.has_parent(self, ctx, pos)?
            && self
            .generics
            .iter()
            .flat_map(|n| other.generics.iter().map(move |o| n.is_superset_of(o, ctx, pos)))
            .collect::<Result<Vec<bool>, _>>()?
            .iter()
            .all(|b| *b))
    }
}

impl StringName {
    pub fn new(lit: &str, generics: &[Name]) -> StringName {
        StringName { name: String::from(lit), generics: Vec::from(generics) }
    }

    pub fn empty() -> StringName {
        StringName::new("()", &[])
    }

    pub fn substitute(
        &self,
        generics: &HashMap<Name, Name>,
        pos: Position,
    ) -> TypeResult<StringName> {
        if let Some(name) = generics.get(&Name::from(self)) {
            let msg = format!("{} is not a DirectName", name);
            let string_names = name.as_direct(&msg, pos)?;
            if string_names.len() > 1 {
                let msg = format!("Cannot substitute type union {}", name);
                return Err(vec![TypeErr::new(pos, &msg)]);
            }

            if let Some(string_name) = string_names.iter().next() {
                return Ok(string_name.clone());
            }

            let msg = format!("{} incorrect DirectName", name);
            Err(vec![TypeErr::new(pos, &msg)])
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
