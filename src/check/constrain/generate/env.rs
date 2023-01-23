use std::collections::{HashMap, HashSet};

use crate::check::constrain::constraint::builder::{format_var_map, VarMapping};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;

#[derive(Clone, Debug, Default)]
pub struct Environment {
    pub in_loop: bool,
    pub in_fun: bool,
    pub is_expr: bool,
    pub is_def_mode: bool,
    pub is_destruct_mode: bool,
    pub return_type: Option<Expected>,

    pub raises_caught: HashSet<TrueName>,

    pub class: Option<StringName>,

    pub unassigned: HashSet<String>,

    pub vars: HashMap<String, HashSet<(bool, Expected)>>,
    pub var_mapping: VarMapping,
}

impl Environment {
    /// Specify that we are in a class
    pub fn in_class(&self, class_name: &StringName) -> Environment {
        Environment { class: Some(class_name.clone()), ..self.clone() }
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

    pub fn is_destruct_mode(&self, is_destruct_mode: bool) -> Self {
        Environment { is_destruct_mode, ..self.clone() }
    }

    pub fn is_expr(&self, is_expr: bool) -> Environment {
        Environment { is_expr, ..self.clone() }
    }

    pub fn override_mapping(&self, var: &str, mapping: usize) -> Self {
        let mut var_mapping = self.var_mapping.clone();
        var_mapping.insert(String::from(var), mapping);
        Environment { var_mapping, ..self.clone() }
    }

    /// Insert a variable.
    ///
    /// If the var was previously defined, it is renamed, and the rename mapping is stored.
    /// In future, if we get a variable, if it was renamed, the mapping is returned instead.
    pub fn insert_var(&self, mutable: bool, var: &str, expect: &Expected, var_mapping: &VarMapping) -> Environment {
        let expected_set = vec![(mutable, expect.clone())].into_iter().collect::<HashSet<_>>();
        let mut vars = self.vars.clone();

        let offset = if let Some(offset) = self.var_mapping.get(var) {
            *offset + 1
        } else if let Some(offset) = var_mapping.get(var) {
            *offset
        } else {
            0_usize
        };

        let mut var_mappings = self.var_mapping.clone();
        var_mappings.insert(String::from(var), offset);

        let mapped_var = format_var_map(var, &offset);
        trace!("Inserted {var} in environment: {var} => {mapped_var} ({expect})");
        vars.insert(mapped_var, expected_set);
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
    pub fn get_var(&self, var: &str, var_mapping: &VarMapping) -> Option<HashSet<(bool, Expected)>> {
        let var_name = if let Some(offset) = self.var_mapping.get(var) {
            format_var_map(var, offset)
        } else if let Some(offset) = var_mapping.get(var) {
            format_var_map(var, offset)
        } else {
            String::from(var)
        };

        self.vars.get(&var_name).cloned()
    }

    pub fn remove_var(&self, var: &str) -> Self {
        let mut vars = self.vars.clone();
        vars.remove(var);
        Environment { vars, ..self.clone() }
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

    /// Union with unassigned of other.
    pub fn union(&self, other: &Environment) -> Environment {
        let unassigned = self.unassigned.union(&other.unassigned);
        Environment { unassigned: unassigned.cloned().collect(), ..self.clone() }
    }

    /// Intersection with unassigned of other.
    pub fn intersection(&self, other: &Environment) -> Environment {
        let unassigned = self.unassigned.intersection(&other.unassigned);
        Environment { unassigned: unassigned.cloned().collect(), ..self.clone() }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::check::constrain::generate::env::Environment;

    #[test]
    fn union_unassigned() {
        let (env1, env2) = (Environment::default(), Environment::default());
        let env1 = env1.with_unassigned(HashSet::from([String::from("a")]));
        let env2 = env2.with_unassigned(HashSet::from([String::from("a")]));
        assert_eq!(env1.unassigned.len(), 1);

        let env1 = env1.assigned_to(&String::from("a"));
        assert_eq!(env1.unassigned.len(), 0);
        assert_eq!(env2.unassigned.len(), 1);

        let env3 = env1.union(&env2);
        assert!(env3.unassigned.contains(&String::from("a")));
        assert_eq!(env3.unassigned.len(), 1);
    }
}
