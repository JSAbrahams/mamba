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
            let generics: Vec<Core> = self.names.iter().map(|name| name.to_py(imp)).collect();
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

/// Produce type with alphabetized generics, ensuring that for any two equal sets of generics the
/// order in which they are given is always the same.
///
/// - Ignores capitalization of generics.
/// - Types with no literal are at the front.
fn core_type(lit: &str, generics: &[Core]) -> Core {
    let generics: Vec<Core> = generics.iter()
        .map(|core| match &core {
            Core::Type { lit, generics } => core_type(lit, generics),
            _ => core.clone()
        })
        .sorted_by_key(|c| match c {
            Core::Type { lit, .. } => lit.clone(),
            _ => String::new()
        }).collect();

    Core::Type { lit: String::from(lit), generics }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::check::name::{Name, Nullable, TupleCallable};
    use crate::check::name::string_name::StringName;
    use crate::generate::ast::node::Core;
    use crate::generate::convert::state::Imports;
    use crate::generate::name::{core_type, ToPy};

    #[test]
    fn alphabetize_ids() {
        let generics = vec!["z", "b", "e"];
        let generics: Vec<Core> =
            generics.iter().map(|id| Core::Id { lit: String::from(*id) }).collect();

        let core = core_type("something", &generics);
        assert_eq!(
            core,
            Core::Type {
                lit: String::from("something"),
                generics: vec![
                    Core::Id { lit: String::from("b") },
                    Core::Id { lit: String::from("e") },
                    Core::Id { lit: String::from("z") },
                ],
            }
        )
    }

    #[test]
    fn alphabetize_generics() {
        let generics = vec!["z", "b"];
        let mut generics: Vec<Core> =
            generics.iter().map(|id| Core::Id { lit: String::from(*id) }).collect();

        let generic = Core::Type {
            lit: String::from("H"),
            generics: vec![
                Core::Id { lit: String::from("k") },
                Core::Id { lit: String::from("a") },
            ],
        };
        generics.push(generic);

        let generic = Core::Type {
            lit: String::from("C"),
            generics: vec![
                Core::Id { lit: String::from("l") },
                Core::Id { lit: String::from("p") },
            ],
        };
        generics.push(generic);

        let core = core_type("something", &generics);
        assert_eq!(
            core,
            Core::Type {
                lit: String::from("something"),
                generics: vec![
                    Core::Id { lit: String::from("b") },
                    Core::Type {
                        lit: String::from("C"),
                        generics: vec![
                            Core::Id { lit: String::from("l") },
                            Core::Id { lit: String::from("p") },
                        ],
                    },
                    Core::Type {
                        lit: String::from("H"),
                        generics: vec![
                            Core::Id { lit: String::from("a") },
                            Core::Id { lit: String::from("k") },
                        ],
                    },
                    Core::Id { lit: String::from("z") },
                ],
            }
        )
    }

    #[test]
    fn union_nullable_tuple_callable() {
        let (a, b) = (StringName::from("a"), StringName::from("b"));
        let (c, d, e) = (StringName::from("c"), StringName::from("d"), StringName::from("e"));

        let tuple = Name::tuple(&[Name::from(&a), Name::from(&b)]);
        let callable = Name::callable(&[Name::from(&c), Name::from(&d)], &Name::from(&e));
        let nullable = callable.as_nullable();

        let name = Name::from(&HashSet::from([tuple, nullable]));

        let mut imports = Imports::new();
        let core_name = name.to_py(&mut imports);

        let import = vec!["Callable", "Optional", "Tuple", "Union"];
        let import = import.iter().map(|ty| Core::Id { lit: String::from(*ty) }).collect();
        let core = Box::from(Core::Id { lit: String::from("typing") });
        let import = Core::Import { from: Some(core), import, alias: vec![] };
        assert!(imports.imports().contains(&import));

        assert_eq!(
            core_name,
            Core::Type {
                lit: String::from("Union"),
                generics: vec![
                    Core::Type {
                        lit: String::from("Optional"),
                        generics: vec![Core::Type {
                            lit: String::from("Callable"),
                            generics: vec![
                                Core::Type {
                                    lit: String::from(""),
                                    generics: vec![
                                        Core::Id { lit: String::from("c") },
                                        Core::Id { lit: String::from("d") },
                                    ],
                                },
                                Core::Id { lit: String::from("e") },
                            ],
                        }],
                    },
                    Core::Type {
                        lit: String::from("Tuple"),
                        generics: vec![
                            Core::Id { lit: String::from("a") },
                            Core::Id { lit: String::from("b") },
                        ],
                    },
                ],
            }
        )
    }
}
