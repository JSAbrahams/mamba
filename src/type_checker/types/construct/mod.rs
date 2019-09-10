use crate::type_checker::context::type_name::TypeName;

// TODO look up range implemention of Python
pub const RANGE: TypeName = TypeName::Single { lit: String::from("range"), generics: vec![] };
