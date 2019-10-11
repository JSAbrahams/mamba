use crate::common::position::Position;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::HashMap;

pub mod expression_type;
pub mod infer_type;
pub mod state;

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

    // TODO use mutable
    pub fn insert(
        self,
        var: &str,
        mutable: bool,
        expr_ty: &ExpressionType
    ) -> TypeResult<Environment> {
        let mut variables = self.variables.clone();
        variables.insert(String::from(var), expr_ty.clone());
        Ok(Environment { variables })
    }

    pub fn union(self, env: Environment) -> Environment {
        let mut variables = self.variables;
        variables.extend(env.variables);
        Environment { variables }
    }

    // TODO change to intersection
    pub fn intersection(self, env: Environment) -> Environment {
        let mut variables = self.variables;
        variables.extend(env.variables);
        Environment { variables }
    }
}
