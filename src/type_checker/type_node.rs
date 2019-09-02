use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;

#[derive(Debug)]
pub struct Type {
    name:      String,
    generics:  Vec<Type>,
    fields:    Vec<Field>,
    functions: Vec<Function>
}

impl Type {
    pub fn new(ty: &str) -> Type {
        Type {
            name:      String::from(ty),
            generics:  vec![],
            fields:    vec![],
            functions: vec![]
        }
    }
}
