use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter};
use std::iter::FromIterator;

use crate::common::delimit::comma_delimited;

pub mod generic;
pub mod python;

/// Name is the actual name of a Function, Field, or generic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Name {
    Single(String, Vec<Name>),
    Tuple(Vec<Name>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameUnion {
    pub names: HashSet<Name>
}

impl Display for NameUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            if self.names.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", comma_delimited(self.names))
            }
        )
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}{}",
            self.name,
            if self.generics.is_empty() {
                String::new()
            } else {
                format!("[{}]", comma_delimited(self.generics))
            }
        )
    }
}

impl From<&Name> for NameUnion {
    fn from(name: &Name) -> Self { NameUnion { names: HashSet::from_iter(vec![name].iter()) } }
}

impl From<&str> for NameUnion {
    fn from(name: &str) -> Self { NameUnion::from(&Name::from(name)) }
}

impl From<&str> for Name {
    fn from(name: &str) -> Self { Name::Single(String::from(name), vec![]) }
}

impl NameUnion {
    pub fn new(names: &[Name]) -> NameUnion { NameUnion { names: HashSet::from_iter(names) } }
}

impl Name {
    pub fn empty() -> Name { Name::Single(String::from("()"), vec![]) }

    pub fn substitute(&self, generics: &HashMap<String, Name>) -> Name {
        match &self {
            Name::Single(name, gens) =>
                if let Some(name) = generics.get(name) {
                    name.clone()
                } else {
                    Name::Single(
                        name.clone(),
                        gens.iter().map(|generic| generic.substitue(generics)).collect()
                    )
                },
            Name::Tuple(names) =>
                Name::Tuple(names.iter().map(|n| n.substitute(generics)).collect()),
        }
    }
}
