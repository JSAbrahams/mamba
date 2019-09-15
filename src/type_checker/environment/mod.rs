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

    pub fn add_field(&self) -> Environment {
        let new_field = Field { name: "".to_string(), ty: None };

        let mut fields = self.fields.clone();
        fields.insert(new_field.name, new_field.clone());

        Environment { functions: self.functions.clone(), fields }
    }

    pub fn union(&self, env: Environment) -> Environment {
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

    pub fn intersection(&self, env: Environment) -> Environment { unimplemented!() }
}
