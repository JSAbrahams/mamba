use std::collections::HashMap;

use crate::type_checker::environment::field::Field;
use crate::type_checker::environment::function::Function;

pub mod field;
pub mod function;
pub mod function_arg;
pub mod ty;
pub mod type_name;

#[derive(Clone)]
pub struct Environment {
    functions: HashMap<String, Function>,
    fields:    HashMap<String, Field>
}

impl Environment {
    pub fn new() -> Environment {
        Environment { functions: HashMap::new(), fields: HashMap::new() }
    }

    pub fn union(&self, env: &Environment) -> Environment {
        let mut functions = self.functions.clone();
        let mut fields = self.fields.clone();

        env.functions.iter().for_each(|(k, v)| {
            functions.insert(k.clone(), v.clone());
        });
        env.fields.iter().for_each(|(k, v)| {
            fields.insert(k.clone(), v.clone());
        });

        Environment { functions, fields }
    }
}
