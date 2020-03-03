use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use crate::check::constrain::constraint::expected::Expect::Raises;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::context::arg::SELF;
use crate::check::context::name::NameUnion;
use crate::common::position::Position;

#[derive(Clone, Debug)]
pub struct Environment {
    pub in_loop: bool,
    pub last_stmt_in_function: bool,
    pub is_define_mode: bool,
    pub return_type: Option<Expected>,
    pub raises: Option<Expected>,
    pub class_type: Option<Expect>,
    pub vars: HashMap<String, HashSet<(bool, Expected)>>
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            in_loop: false,
            is_define_mode: false,
            last_stmt_in_function: false,
            return_type: None,
            raises: None,
            class_type: None,
            vars: HashMap::new()
        }
    }
}

impl Environment {
    /// Specify that we are in a class
    ///
    /// This adds a self variable with the class expected, and class_type is set
    /// to the expected class type.
    pub fn in_class(&self, class: &Expected) -> Environment {
        let env = self.insert_var(false, &String::from(SELF), class);
        Environment { class_type: Some(class.expect.clone()), ..env }
    }

    /// Sets environment into define mode.
    ///
    /// Causes all identifiers to be treated as definitions.
    pub fn define_mode(&self, is_define_mode: bool) -> Environment {
        Environment { is_define_mode, ..self.clone() }
    }

    /// Insert a variable.
    ///
    /// If it has a previous expected type then this is overwritten
    pub fn insert_var(&self, mutable: bool, var: &str, expect: &Expected) -> Environment {
        let mut vars = self.vars.clone();
        let expected_set = HashSet::from_iter(vec![(mutable, expect.clone())].into_iter());
        vars.insert(String::from(var), expected_set);
        Environment { vars, ..self.clone() }
    }

    /// Insert raises.
    pub fn insert_raises(&self, raises: &NameUnion, pos: &Position) -> Environment {
        if raises.is_empty() {
            self.clone()
        } else {
            let raises = Expected::new(pos, &Raises { name: raises.clone() });
            Environment { raises: Some(raises), ..self.clone() }
        }
    }

    /// Specify that we are in a loop.
    pub fn in_loop(&self) -> Environment { Environment { in_loop: true, ..self.clone() } }

    /// Specify the return type of function body.
    pub fn return_type(&self, return_type: &Expected) -> Environment {
        Environment {
            return_type: Some(return_type.clone()),
            last_stmt_in_function: true,
            ..self.clone()
        }
    }

    /// Gets a variable.
    ///
    /// Is Some, Vector wil usually contain only one expected.
    /// It can contain multiple if the environment was unioned or intersected at
    /// one point.
    pub fn get_var(&self, var: &str) -> Option<HashSet<(bool, Expected)>> {
        self.vars.get(var).cloned()
    }

    /// Union between two environments
    ///
    /// Combines all variables.
    pub fn union(&self, other: &Environment) -> Environment {
        let mut vars = self.vars.clone();
        for (key, other_set) in &other.vars {
            if let Some(this_set) = vars.get(key) {
                let new_set = this_set.union(&other_set).cloned().collect();
                vars.insert(key.clone(), new_set);
            } else {
                vars.insert(key.clone(), other_set.clone());
            }
        }
        Environment { vars, ..self.clone() }
    }

    /// Intersection between two environments.
    ///
    /// If both environments contain the same variable, variable gets assigned
    /// both the expected. Variables that are only present in one of the
    /// environments are discarded.
    ///
    /// Only intersect vars, all other fields of other environment are
    /// discarded.
    pub fn intersect(&self, other: &Environment) -> Environment {
        let keys = self.vars.keys().filter(|key| other.vars.contains_key(*key));
        let mut vars = HashMap::new();
        for key in keys {
            match (self.vars.get(key), other.vars.get(key)) {
                (Some(l_exp), Some(r_exp)) => {
                    let union = l_exp.union(r_exp);
                    vars.insert(String::from(key), HashSet::from_iter(union.cloned()));
                }
                (Some(exp), None) | (None, Some(exp)) => {
                    vars.insert(String::from(key), exp.clone());
                }
                _ => {}
            }
        }

        Environment { vars, ..self.clone() }
    }
}
