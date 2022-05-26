use crate::check::context::clss::concrete_to_python;
use crate::check::name::Name;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::generate::ast::node::Core;
use crate::generate::convert::state::Imports;

pub trait ToPy {
    fn to_py(&self, imp: &mut Imports) -> Core;
}

impl ToPy for Name {
    fn to_py(&self, imp: &mut Imports) -> Core {
        if self.names.len() > 1 {
            imp.add_from_import("typing", "Union");
            let names = self.names.iter().map(|name| name.to_py(imp)).collect();
            Core::Type { lit: String::from("Union"), generics: names }
        } else if let Some(name) = self.names.iter().next() {
            name.to_py(imp)
        } else {
            Core::Empty
        }
    }
}

impl ToPy for TrueName {
    fn to_py(&self, imp: &mut Imports) -> Core {
        self.variant.to_py(imp)
    }
}

impl ToPy for NameVariant {
    fn to_py(&self, imp: &mut Imports) -> Core {
        match &self {
            NameVariant::Single(name) => name.to_py(imp),
            NameVariant::Tuple(names) => {
                imp.add_from_import("typing", "Tuple");
                let names = names.iter().map(|name| name.to_py(imp)).collect();
                Core::Type { lit: String::from("Tuple"), generics: names }
            }
            NameVariant::Fun(args, ret) => {
                imp.add_from_import("typing", "Callable");
                let args = args.iter().map(|name| name.to_py(imp)).collect();
                let ret = ret.to_py(imp);

                Core::Type {
                    lit: String::from("Callable"),
                    generics: vec![Core::Type { lit: String::new(), generics: args }, ret],
                }
            }
        }
    }
}

impl ToPy for StringName {
    fn to_py(&self, imp: &mut Imports) -> Core {
        let lit = concrete_to_python(&self.name);
        let generics = self.generics.iter().map(|name| name.to_py(imp)).collect();
        Core::Type { lit, generics }
    }
}
