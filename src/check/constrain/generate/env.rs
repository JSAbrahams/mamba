use std::collections::{HashMap, HashSet};

use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::context::arg::SELF;
use crate::check::name;
use crate::check::name::true_name::TrueName;

#[derive(Clone, Debug, Default)]
pub struct Environment {
    pub in_loop: bool,
    pub is_expr: bool,
    pub is_def_mode: bool,
    pub return_type: Option<Expected>,

    pub raises_caught: HashSet<TrueName>,

    pub class_type: Option<Expect>,
    pub var_mappings: HashMap<String, String>,
    pub unassigned: HashSet<String>,
    temp_type: usize,
    vars: HashMap<String, HashSet<(bool, Expected)>>,
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
    pub fn is_def_mode(&self, is_def_mode: bool) -> Environment {
        Environment { is_def_mode, ..self.clone() }
    }

    pub fn is_expr(&self, is_expr: bool) -> Environment {
        Environment { is_expr, ..self.clone() }
    }

    /// Insert a variable.
    ///
    /// If the var was previously defined, it is renamed, and the rename mapping is stored.
    /// In future, if we get a variable, if it was renamed, the mapping is returned instead.
    pub fn insert_var(&self, mutable: bool, var: &str, expect: &Expected) -> Environment {
        let expected_set = vec![(mutable, expect.clone())].into_iter().collect::<HashSet<_>>();
        let (mut vars, mut var_mappings) = (self.vars.clone(), self.var_mappings.clone());

        // Never shadow self
        let var = if vars.contains_key(var) && var != SELF {
            let mut offset = 0;
            let mut new_var = format!("{}@{}", var, offset);
            while self.vars.contains_key(&new_var) {
                offset += 1;
                new_var = format!("{var}@{offset}");
            }

            var_mappings.insert(String::from(var), new_var.clone());
            new_var
        } else {
            String::from(var)
        };

        vars.insert(var, expected_set);
        Environment { vars, var_mappings, ..self.clone() }
    }

    /// Insert raises which are properly handled.
    ///
    /// Appends to current set.
    pub fn raises_caught(&self, raises: &HashSet<TrueName>) -> Environment {
        let raises_caught = self.raises_caught.union(raises).cloned().collect();
        Environment { raises_caught, ..self.clone() }
    }

    /// Specify that we are in a loop.
    pub fn in_loop(&self) -> Environment {
        Environment { in_loop: true, ..self.clone() }
    }

    /// Specify the return type of function body.
    pub fn return_type(&self, return_type: &Expected) -> Environment {
        Environment { return_type: Some(return_type.clone()), ..self.clone() }
    }

    /// Gets a variable.
    ///
    /// Is Some, Vector wil usually contain only one expected.
    /// It can contain multiple if the environment was unioned or intersected at
    /// one point.
    ///
    /// If the variable was mapped to another variable at one point due to a naming conflict,
    /// the mapped to variable is returned to instead.
    /// In other words, what the variable was mapped to.
    /// This is useful for detecting shadowing.
    ///
    /// Return true variable [TrueName], whether it's mutable and it's expected value
    pub fn get_var(&self, var: &str) -> Option<HashSet<(bool, Expected)>> {
        for (old, new) in &self.var_mappings {
            if old == var {
                return self.get_var(new);
            }
        }

        self.vars.get(var).cloned()
    }

    /// Union between two environments
    ///
    /// Combines all variables.
    ///
    /// Variable mappings combined.
    /// If mapping occurs in both environments, then those of this environment taken.
    pub fn union(&self, other: &Environment) -> Environment {
        let mut vars = self.vars.clone();
        for (key, other_set) in &other.vars {
            if let Some(this_set) = vars.get(key) {
                let new_set = this_set.union(other_set).cloned().collect();
                vars.insert(key.clone(), new_set);
            } else {
                vars.insert(key.clone(), other_set.clone());
            }
        }

        let mut var_mappings = self.var_mappings.clone();
        for (key, value) in &other.var_mappings {
            var_mappings.insert(key.clone(), value.clone());
        }

        Environment { vars, var_mappings, ..self.clone() }
    }

    /// Intersection between two environments.
    ///
    /// If both environments contain the same variable, variable gets assigned
    /// both the expected. Variables that are only present in one of the
    /// environments are discarded.
    ///
    /// Only intersect vars, all other fields of other environment are
    /// discarded.
    ///
    /// Var mappings from this environment preserved which also occur in the other environment.
    /// However, mappings of other environment preserved.
    pub fn intersect(&self, other: &Environment) -> Environment {
        let keys = self.vars.keys().filter(|key| other.vars.contains_key(*key));
        let mut vars = HashMap::new();
        for key in keys {
            match (self.vars.get(key), other.vars.get(key)) {
                (Some(l_exp), Some(r_exp)) => {
                    let union = l_exp.union(r_exp);
                    vars.insert(String::from(key), union.cloned().collect::<HashSet<_>>());
                }
                (Some(exp), None) | (None, Some(exp)) => {
                    vars.insert(String::from(key), exp.clone());
                }
                _ => {}
            }
        }

        let to_remove: Vec<String> = self
            .var_mappings
            .iter()
            .filter(|(key, _)| other.var_mappings.contains_key(*key))
            .map(|(key, _)| key.clone())
            .collect();

        let mut var_mappings = self.var_mappings.clone();
        for key in &to_remove {
            var_mappings.remove(key);
        }

        Environment { vars, var_mappings, ..self.clone() }
    }

    /// Get a name for a temporary type.
    ///
    /// Useful for when we don't know what a type should be during the generation stage.
    /// The unification stage should then identify these.
    pub fn temp_var(&self) -> (String, Environment) {
        (
            format!("{}{}", name::TEMP, self.temp_type),
            Environment { temp_type: self.temp_type + 1, ..self.clone() },
        )
    }

    /// Denote a set of variables which should be assigned to at some point.
    pub fn with_unassigned(&self, unassigned: HashSet<String>) -> Environment {
        Environment { unassigned, ..self.clone() }
    }

    /// Denote that a variable was assigned to by removing it from the set of variables which
    /// should be assigned to.
    ///
    /// If not in environment, then nothing happens.
    pub fn assigned_to(&self, var: &String) -> Environment {
        let mut unassigned = self.unassigned.clone();
        unassigned.remove(var);
        Environment { unassigned, ..self.clone() }
    }
}
