use crate::common::position::Position;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::{HashMap, HashSet};

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
        self.variables.get(var).cloned().ok_or(vec![TypeErr::new(pos, "Undefined variable")])
    }

    pub fn union(self, _: Environment) -> Environment { unimplemented!() }

    pub fn intersection(self, _: Environment) -> Environment { unimplemented!() }
}
