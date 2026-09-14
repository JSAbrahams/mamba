use std::collections::{HashMap, HashSet};

use log::trace;

use crate::check::constrain::constraint::builder::{format_var_map, VarMapping};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::context::arg::SELF;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;

#[derive(Clone, Debug, Default)]
pub struct Environment {
    pub in_loop: bool,
    pub in_fun: bool,
    /// Whether we are in the body of a `pure` function, where the purity rules apply.
    pub in_pure: bool,
    /// Mutable variables defined outside the current `pure` function's body.
    ///
    /// A pure function may neither read nor assign to these, since both make its result
    /// depend on, or leak into, state its arguments do not cover.
    pub outer_mut: HashSet<String>,
    /// Names in scope when the current `pure` function's body began, so its arguments and
    /// everything enclosing it.
    ///
    /// Anything not in here was defined by the body itself, and is destroyed on exit, so it
    /// may be used freely.
    pub pure_nonlocal: HashSet<String>,
    pub is_expr: bool,
    pub is_def_mode: bool,
    pub is_destruct_mode: bool,
    pub return_type: Option<Expected>,

    pub raises_caught: HashSet<TrueName>,

    pub class: Option<StringName>,

    pub vars: HashMap<String, HashSet<(bool, Expected)>>,
    pub var_mapping: VarMapping,
}

impl Environment {
    /// Specify that we are in a class
    pub fn in_class(&self, class_name: &StringName) -> Environment {
        Environment {
            class: Some(class_name.clone()),
            ..self.clone()
        }
    }

    pub fn in_fun(&self, in_fun: bool) -> Environment {
        Environment {
            in_fun,
            ..self.clone()
        }
    }

    /// Enter the body of a `pure` function.
    ///
    /// `outer` is the scope the function is defined in. Every mutable variable there is off
    /// limits inside the body, since reading or assigning it makes the result depend on, or
    /// leak into, state the arguments do not cover. The function's own arguments are not
    /// outer, so a `mut` argument may still be assigned to.
    ///
    /// `self` is excluded for the same reason: a class body binds it, so it would otherwise
    /// look enclosing, when it is really this method's own argument. The `mut self` rule
    /// covers it instead.
    pub fn in_pure(&self, outer: &Environment) -> Environment {
        let outer_mut = outer
            .vars
            .iter()
            .filter(|(var, _)| *var != SELF)
            .filter(|(_, exps)| exps.iter().any(|(mutable, _)| *mutable))
            .map(|(var, _)| var.clone())
            .collect();

        Environment {
            in_pure: true,
            outer_mut,
            pure_nonlocal: self.vars.keys().cloned().collect(),
            ..self.clone()
        }
    }

    /// Sets environment into define mode.
    ///
    /// Causes all identifiers to be treated as definitions.
    pub fn is_def_mode(&self, is_def_mode: bool) -> Environment {
        Environment {
            is_def_mode,
            ..self.clone()
        }
    }

    pub fn is_destruct_mode(&self, is_destruct_mode: bool) -> Self {
        Environment {
            is_destruct_mode,
            ..self.clone()
        }
    }

    pub fn is_expr(&self, is_expr: bool) -> Environment {
        Environment {
            is_expr,
            ..self.clone()
        }
    }

    pub fn override_mapping(&self, var: &str, mapping: usize) -> Self {
        let mut var_mapping = self.var_mapping.clone();
        var_mapping.insert(String::from(var), mapping);
        Environment {
            var_mapping,
            ..self.clone()
        }
    }

    /// Insert a variable.
    ///
    /// If the var was previously defined, it is renamed, and the rename mapping is stored.
    /// In future, if we get a variable, if it was renamed, the mapping is returned instead.
    pub fn insert_var(
        &self,
        mutable: bool,
        var: &str,
        expect: &Expected,
        var_mapping: &VarMapping,
    ) -> Environment {
        let expected_set = vec![(mutable, expect.clone())]
            .into_iter()
            .collect::<HashSet<_>>();
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
        Environment {
            vars,
            var_mapping: var_mappings,
            ..self.clone()
        }
    }

    /// Insert raises which are properly handled.
    ///
    /// Appends to current set.
    pub fn raises_caught(&self, raises: &HashSet<TrueName>) -> Environment {
        let raises_caught = self.raises_caught.union(raises).cloned().collect();
        Environment {
            raises_caught,
            ..self.clone()
        }
    }

    /// Specify that we are in a loop.
    pub fn in_loop(&self) -> Environment {
        Environment {
            in_loop: true,
            ..self.clone()
        }
    }

    /// Specify the return type of function body.
    pub fn return_type(&self, return_type: &Expected) -> Environment {
        Environment {
            return_type: Some(return_type.clone()),
            ..self.clone()
        }
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
    pub fn get_var(
        &self,
        var: &str,
        var_mapping: &VarMapping,
    ) -> Option<HashSet<(bool, Expected)>> {
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
        Environment {
            vars,
            ..self.clone()
        }
    }
}
