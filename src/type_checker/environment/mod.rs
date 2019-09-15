use crate::type_checker::environment::expression_type::ExpressionType;
use std::collections::HashMap;

pub mod expression_type;
pub mod state;

#[derive(Clone)]
pub struct Environment {
    variables: HashMap<String, ExpressionType>
}

impl Environment {
    pub fn new() -> Environment { Environment { variables: HashMap::new() } }

    pub fn union(&self, env: Environment) -> Environment { unimplemented!() }

    pub fn intersection(&self, env: Environment) -> Environment { unimplemented!() }
}
