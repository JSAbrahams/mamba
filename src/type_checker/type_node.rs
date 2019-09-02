use crate::type_checker::context::function::Function;

pub struct Type {
    name:      String,
    generics:  Vec<Type>,
    functions: Vec<Function>
}

impl Type {
    pub fn new(ty: &str) -> Type {
        Type { name: String::from(ty), generics: vec![], functions: vec![] }
    }
}
