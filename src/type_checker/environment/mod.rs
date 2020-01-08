use crate::common::position::Position;
use crate::type_checker::context::function_arg;
use crate::type_checker::environment::state::State;
use crate::type_checker::infer_type::expression::ExpressionType;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::{HashMap, HashSet};

pub mod name;
pub mod state;

// TODO use name in lookup functions

#[derive(Clone, Debug)]
pub struct Environment {
    pub state: State,
    pub vars:  HashSet<String>,
    variables: HashMap<String, (bool, ExpressionType)>
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            state:     State::default(),
            vars:      HashSet::new(),
            variables: HashMap::new()
        }
    }
}

impl Environment {
    pub fn lookup_indirect(&self, var: &str, pos: &Position) -> TypeResult<(bool, ExpressionType)> {
        self.variables
            .get(var)
            .cloned()
            .ok_or_else(|| vec![TypeErr::new(pos, &format!("Undefined variable: {}", var))])
    }

    pub fn lookup(&self, var: &str, pos: &Position) -> TypeResult<ExpressionType> {
        self.variables
            .get(var)
            .cloned()
            .map(|(_, expr_ty)| expr_ty)
            .ok_or_else(|| vec![TypeErr::new(pos, &format!("Undefined variable: {}", var))])
    }

    pub fn in_class(&self, mutable: bool, class: &ExpressionType) -> Environment {
        let mut variables = self.variables.clone();
        variables.insert(String::from(function_arg::concrete::SELF), (mutable, class.clone()));

        let state = self.state.in_class(&TypeName::from(class));
        Environment { state, vars: self.vars.clone(), variables }
    }

    pub fn new_state(&self, state: &State) -> Self {
        Environment { state: state.clone(), ..self.clone() }
    }

    pub fn remove(&mut self, var: &str) -> Option<(bool, ExpressionType)> {
        self.variables.remove(var)
    }

    pub fn insert(&mut self, var: &str, mutable: bool, expr_ty: &ExpressionType) {
        self.variables.insert(String::from(var), (mutable, expr_ty.clone()));
    }

    // TODO implement properly
    pub fn difference(self, env: Environment) -> Environment {
        let mut variables = self.variables;
        variables.extend(env.variables);
        Environment { variables, ..self }
    }
}
