use crate::type_checker::environment::expression_type::ExpressionTypes;
use std::collections::HashMap;

pub mod actual_type;
pub mod expression_type;
pub mod state;

#[derive(Clone, Debug)]
pub struct Environment {
    variables: HashMap<String, ExpressionTypes>
}

impl Environment {
    pub fn new() -> Environment { Environment { variables: HashMap::new() } }

    pub fn union(self, _: Environment) -> Environment { unimplemented!() }

    pub fn intersection(self, _: Environment) -> Environment { unimplemented!() }
}
