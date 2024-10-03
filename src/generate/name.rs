use itertools::Itertools;

use crate::check::context::clss;
use crate::check::context::clss::concrete_to_python;
use crate::check::context::clss::python::{ANY, CALLABLE, TUPLE, UNION};
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::check::name::{Empty, Name, Nullable, Union};
use crate::generate::ast::node::Core;
use crate::generate::convert::state::Imports;

pub trait ToPy {
    fn to_py(&self, imp: &mut Imports) -> Core;
}

impl ToPy for Name {
    fn to_py(&self, imp: &mut Imports) -> Core {
        if self.names.len() > 1 {
            imp.add_from_import("typing", UNION);
            let generics: Vec<Name> = self.names.iter().sorted().map(Name::from).collect();
            core_type(UNION, &generics, imp)
        } else if let Some(name) = self.names.iter().next() {
            name.to_py(imp)
        } else {
            Core::Empty
        }
    }
}

impl ToPy for TrueName {
    fn to_py(&self, imp: &mut Imports) -> Core {
        if self.is_nullable() {
            imp.add_from_import("typing", "Optional");
            core_type("Optional", &[Name::from(&self.variant)], imp)
        } else {
            self.variant.to_py(imp)
        }
    }
}

impl ToPy for StringName {
    fn to_py(&self, imp: &mut Imports) -> Core {
        match self.name.as_str() {
            clss::UNION => self
                .generics
                .iter()
                .sorted()
                .fold(Name::empty(), |acc, n| acc.union(n))
                .to_py(imp),
            clss::TUPLE => {
                imp.add_from_import("typing", TUPLE);
                core_type(TUPLE, &self.generics, imp)
            }
            clss::CALLABLE => {
                imp.add_from_import("typing", CALLABLE);
                let args = self.generics.get(0).cloned().unwrap_or_else(Name::empty);
                let ret = self.generics.get(1).cloned().unwrap_or_else(Name::empty);
                core_type(CALLABLE, &[args, ret], imp)
            }
            other => {
                if other == clss::ANY {
                    imp.add_from_import("typing", ANY);
                }

                let lit = concrete_to_python(&self.name);
                core_type(&lit, &self.generics, imp)
            }
        }
    }
}

fn core_type(lit: &str, generics: &[Name], imp: &mut Imports) -> Core {
    Core::Type {
        lit: String::from(lit),
        generics: generics.iter().map(|core| core.to_py(imp)).collect(),
    }
}
