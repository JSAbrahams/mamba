use crate::common::position::Position;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::HashMap;

pub mod expression_type;
pub mod infer_type;
pub mod name;
pub mod state;

#[derive(Clone, Debug)]
pub struct Environment {
    variables: HashMap<String, (bool, ExpressionType)>
}

impl Environment {
    pub fn new() -> Environment { Environment { variables: HashMap::new() } }

    pub fn lookup_indirect(&self, var: &str, pos: &Position) -> TypeResult<(bool, ExpressionType)> {
        self.variables
            .get(var)
            .cloned()
            .ok_or(vec![TypeErr::new(pos, &format!("Undefined variable: {}", var))])
    }

    pub fn lookup(&self, var: &str, pos: &Position) -> TypeResult<ExpressionType> {
        self.variables
            .get(var)
            .cloned()
            .map(|(_, expr_ty)| expr_ty)
            .ok_or(vec![TypeErr::new(pos, &format!("Undefined variable: {}", var))])
    }

    pub fn insert(&mut self, var: &str, mutable: bool, expr_ty: &ExpressionType) {
        self.variables.insert(String::from(var), (mutable, expr_ty.clone()));
    }

    // TODO implement properly
    pub fn difference(self, env: Environment) -> Environment {
        let mut variables = self.variables;
        variables.extend(env.variables);
        Environment { variables }
    }
}
