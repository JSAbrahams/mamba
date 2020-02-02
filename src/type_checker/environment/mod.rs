use std::collections::{HashMap, HashSet};

use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::context::function_arg::concrete::SELF;
use std::iter::FromIterator;

pub mod name;

// TODO use name in lookup functions

#[derive(Clone, Debug)]
pub struct Environment {
    pub in_loop:     bool,
    pub return_type: Option<Expected>,
    pub class_type:  Option<Expect>,
    pub vars:        HashMap<String, HashSet<(bool, Expect)>>
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            in_loop:     false,
            return_type: None,
            class_type:  None,
            vars:        HashMap::new()
        }
    }
}

impl Environment {
    /// Specify that we are in a class
    ///
    /// This adds a self variable with the class expected, and class_type is set
    /// to the expected class type.
    pub fn in_class(&self, class: &Expect) -> Environment {
        let env = self.insert_var(false, &String::from(SELF), class);
        Environment { class_type: Some(class.clone()), ..env }
    }

    /// Insert a variable.
    ///
    /// If it has a previous expected type then this is overwritten
    pub fn insert_var(&self, mutable: bool, var: &str, expect: &Expect) -> Environment {
        let mut vars = self.vars.clone();
        let expected_set = HashSet::from_iter(vec![(mutable, expect.clone())].into_iter());
        vars.insert(String::from(var), expected_set);
        Environment { vars, ..self.clone() }
    }

    /// Specify that we are in a loop.
    ///
    /// Useful for checking if a break or continue statement is correctly
    /// placed.
    pub fn in_loop(&self) -> Environment { Environment { in_loop: true, ..self.clone() } }

    /// Specify the return type of function body.
    pub fn return_type(&self, return_type: &Expected) -> Environment {
        Environment { return_type: Some(return_type.clone()), ..self.clone() }
    }

    /// Gets a variable.
    ///
    /// Is Some, Vector wil usually contain only one expected.
    /// It can contain multiple if the environment was unioned or intersected at
    /// one point.
    pub fn get_var(&self, var: &str) -> Option<HashSet<(bool, Expect)>> {
        self.vars.get(var).cloned()
    }

    /// Intersection between two environments.
    ///
    /// If both environments contain the same variable, variable gets assigned
    /// both the expected. Variables that are only present in one of the
    /// environments are discarded.
    pub fn intersect(&self, env: &Environment) -> Environment { Environment { ..self.clone() } }
}
