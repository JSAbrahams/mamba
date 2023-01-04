use std::collections::{HashMap, HashSet};

use crate::check::constrain::constraint::builder::{format_var_map, VarMapping};
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::context::arg::SELF;
use crate::check::name::Name;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::common::position::Position;

#[derive(Clone, Debug, Default)]
pub struct Environment {
    pub in_loop: bool,
    pub in_fun: bool,
    pub is_expr: bool,
    pub is_def_mode: bool,
    pub return_type: Option<Expected>,

    pub raises_caught: HashSet<TrueName>,

    pub class: Option<StringName>,

    pub unassigned: HashSet<String>,
    temp_type: usize,

    pub vars: HashMap<String, HashSet<(bool, Expected)>>,
    pub var_mapping: VarMapping,
}

impl Environment {
    /// Specify that we are in a class
    pub fn in_class(&self, mutable: bool, class_name: &StringName, pos: Position) -> Environment {
        let mut vars = self.vars.clone();
        let exp_class = Expected::new(pos, &Expect::Type { name: Name::from(class_name) });

        vars.insert(String::from(SELF), HashSet::from([(mutable, exp_class)]));
        Environment { class: Some(class_name.clone()), vars, ..self.clone() }
    }

    pub fn in_fun(&self, in_fun: bool) -> Environment {
        Environment { in_fun, ..self.clone() }
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
    pub fn insert_var(&self, mutable: bool, var: &str, expect: &Expected, var_mappings: &VarMapping) -> Environment {
        let expected_set = vec![(mutable, expect.clone())].into_iter().collect::<HashSet<_>>();
        let mut vars = self.vars.clone();

        let offset = if var == SELF {
            0usize // Never shadow self
        } else if let Some(offset) = self.var_mapping.get(var) {
            *offset
        } else if let Some(offset) = var_mappings.get(var) {
            *offset
        } else {
            0usize
        };

        let mut var_mappings = self.var_mapping.clone();
        var_mappings.insert(String::from(var), offset);

        vars.insert(format_var_map(var, &offset), expected_set);
        Environment { vars, var_mapping: var_mappings, ..self.clone() }
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
    /// It can contain multiple if the environment was unioned or intersected at one point.
    ///
    /// If local variable mapping, meaning shadowed locally, then local mapping used to lookup
    /// value.
    /// Else, lookup mapping in global scope.
    /// If not found, use variable directly in lookup.
    ///
    /// Return true variable [TrueName], whether it's mutable and it's expected value.
    pub fn get_var(&self, var: &str, var_mappings: &VarMapping) -> Option<HashSet<(bool, Expected)>> {
        let var_name = if let Some(offset) = self.var_mapping.get(var) {
            format_var_map(var, offset)
        } else if let Some(offset) = var_mappings.get(var) {
            format_var_map(var, offset)
        } else {
            String::from(var)
        };

        self.vars.get(&var_name).cloned()
    }

    /// Union between two environments
    ///
    /// Combines all variables.
    /// Variable mappings are discarded.
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

        Environment { vars, var_mapping: VarMapping::new(), ..self.clone() }
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
    /// Variable mappings are discarded.
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

        Environment { vars, var_mapping: VarMapping::new(), ..self.clone() }
    }

    /// Get a name for a temporary type.
    ///
    /// Useful for when we don't know what a type should be during the generation stage.
    /// The unification stage should then identify these.
    pub fn temp_var(&self) -> (String, Environment) {
        (
            format_var_map("", &(self.temp_type + 1)),
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
