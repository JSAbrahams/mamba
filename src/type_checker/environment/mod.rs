use crate::common::position::Position;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::HashMap;

pub mod expression_type;
pub mod infer_type;
pub mod name;
pub mod state;

// TODO add functions to environment, which may be pure

#[derive(Clone, Debug)]
pub struct Environment {
    variables: HashMap<String, ExpressionType>
}

impl Environment {
    pub fn new() -> Environment { Environment { variables: HashMap::new() } }

    pub fn lookup(&self, var: &str, pos: &Position) -> TypeResult<ExpressionType> {
        self.variables
            .get(var)
            .cloned()
            .ok_or(vec![TypeErr::new(pos, &format!("Undefined variable: {}", var))])
    }

    pub fn insert(&mut self, var: &str, mutable: bool, expr_ty: &ExpressionType) {
        self.variables.insert(String::from(var), expr_ty.clone());
    }

    // TODO implement properly
    pub fn difference(self, env: Environment) -> Environment {
        let mut variables = self.variables;
        variables.extend(env.variables);
        Environment { variables }
    }
}
