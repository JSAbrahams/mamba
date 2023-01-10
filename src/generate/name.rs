use itertools::Itertools;

use crate::check::context::clss;
use crate::check::context::clss::concrete_to_python;
use crate::check::context::clss::python::{CALLABLE, TUPLE, UNION};
use crate::check::name::{Name, Nullable};
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::generate::ast::node::Core;
use crate::generate::convert::state::Imports;

pub trait ToPy {
    fn to_py(&self, imp: &mut Imports) -> Core;
}

impl ToPy for Name {
    fn to_py(&self, imp: &mut Imports) -> Core {
        if self.names.len() > 1 {
            imp.add_from_import("typing", UNION);
            let generics: Vec<Core> = self
                .names
                .iter()
                .sorted_by_key(|name| name.variant.name.clone())
                .map(|name| name.to_py(imp))
                .collect();

            core_type(UNION, &generics)
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
            let generics: Vec<Core> = vec![self.variant.to_py(imp)];
            core_type("Optional", &generics)
        } else {
            self.variant.to_py(imp)
        }
    }
}

impl ToPy for StringName {
    fn to_py(&self, imp: &mut Imports) -> Core {
        match self.name.as_str() {
            clss::TUPLE => {
                imp.add_from_import("typing", TUPLE);
                let generics: Vec<Core> = self.generics.iter().map(|name| name.to_py(imp)).collect();
                core_type(TUPLE, &generics)
            }
            clss::CALLABLE => {
                imp.add_from_import("typing", CALLABLE);
                let args = self.generics.get(0).map_or_else(|| Core::Empty, |args| args.to_py(imp));
                let ret = self.generics.get(1).map_or_else(|| Core::Empty, |name| name.to_py(imp));
                core_type(CALLABLE, &[args, ret])
            }
            _ => {
                let lit = concrete_to_python(&self.name);
                let generics: Vec<Core> = self.generics.iter().map(|name| name.to_py(imp)).collect();
                core_type(&lit, &generics)
            }
        }
    }
}

fn core_type(lit: &str, generics: &[Core]) -> Core {
    Core::Type {
        lit: String::from(lit),
        generics: generics.iter().map(|core| match &core {
            Core::Id { lit } => core_type(lit.as_str(), &[]),
            Core::Type { lit, generics } => core_type(lit, generics),
            _ => core.clone()
        }).collect(),
    }
}
