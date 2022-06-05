use std::collections::BTreeMap;

use itertools::Itertools;

use crate::check::context::clss::concrete_to_python;
use crate::check::name::{Name, Nullable};
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
            let generics: Vec<Core> = self.names.iter().map(|name| name.to_py(imp)).collect();
            core_type("Union", &generics)
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

impl ToPy for NameVariant {
    fn to_py(&self, imp: &mut Imports) -> Core {
        match &self {
            NameVariant::Single(name) => name.to_py(imp),
            NameVariant::Tuple(names) => {
                imp.add_from_import("typing", "Tuple");
                let generics: Vec<Core> = names.iter().map(|name| name.to_py(imp)).collect();
                core_type("Tuple", &generics)
            }
            NameVariant::Fun(args, ret) => {
                imp.add_from_import("typing", "Callable");
                let args = args.iter().map(|name| name.to_py(imp)).collect();
                let ret = ret.to_py(imp);

                let generics = vec![Core::Type { lit: String::new(), generics: args }, ret];
                core_type("Callable", &generics)
            }
        }
    }
}

impl ToPy for StringName {
    fn to_py(&self, imp: &mut Imports) -> Core {
        let lit = concrete_to_python(&self.name);
        let generics: Vec<Core> = self.generics.iter().map(|name| name.to_py(imp)).collect();
        core_type(&lit, &generics)
    }
}

/// Produce type with alphabetized generics, ensuring that for any two equal sets of generics the
/// order in which they are given is always the same.
///
/// - Ignores capitalization of generics.
/// - Types with no literal are at the front.
fn core_type(lit: &str, generics: &[Core]) -> Core {
    let names: BTreeMap<String, Core> = generics
        .iter()
        .map(|core| match core {
            Core::Type { lit, generics } => (lit.clone(), core_type(lit, generics)),
            Core::Id { lit } => (lit.clone(), core.clone()),
            _ => (String::from(""), core.clone()),
        })
        .collect();

    let generics: Vec<Core> = names
        .into_iter()
        .sorted_by_key(|(name, _)| name.clone().to_lowercase())
        .map(|(_, core)| core)
        .collect();

    if generics.is_empty() {
        Core::Id { lit: String::from(lit) }
    } else {
        Core::Type { lit: String::from(lit), generics }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::check::name::{Name, Nullable};
    use crate::check::name::namevariant::NameVariant;
    use crate::check::name::stringname::StringName;
    use crate::check::name::truename::TrueName;
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

        let tuple = NameVariant::Tuple(vec![Name::from(&a), Name::from(&b)]);
        let callable =
            NameVariant::Fun(vec![Name::from(&c), Name::from(&d)], Box::from(Name::from(&e)));
        let nullable = TrueName::from(&callable).as_nullable();

        let name = Name::from(&HashSet::from([Name::from(&tuple), Name::from(&nullable)]));

        let mut imports = Imports::new();
        let core_name = name.to_py(&mut imports);

        macro_rules! assert_import_contains_type {
            ($ty: expr) => {{
                let import = vec![Core::Id { lit: String::from($ty) }];
                let core = Box::from(Core::Id { lit: String::from("typing") });
                let import = Core::Import { from: Some(core), import, alias: vec![] };
                assert!(imports.imports.contains(&import));
            }};
        }

        assert_import_contains_type!("Union");
        assert_import_contains_type!("Callable");
        assert_import_contains_type!("Tuple");
        assert_import_contains_type!("Optional");

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
