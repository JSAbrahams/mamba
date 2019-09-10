use crate::type_checker::context::type_name::TypeName;

pub const INT: TypeName = TypeName::Single { lit: String::from("int"), generics: vec![] };
pub const BOOLEAN: TypeName = TypeName::Single { lit: String::from("bool"), generics: vec![] };
