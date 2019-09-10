use crate::type_checker::context::type_name::TypeName;

pub const INT: TypeName = TypeName { lit: String::from("int"), generics: vec![] };
pub const BOOLEAN: TypeName = TypeName { lit: String::from("bool"), generics: vec![] };
