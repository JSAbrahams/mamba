use python_parser::ast::Expression;

use crate::type_checker::context::generic::type_name::GenericType;

pub const INTEGER: &'static str = "int";
pub const FLOAT: &'static str = "float";
pub const STRING: &'static str = "str";
pub const BOOLEAN: &'static str = "bool";

impl From<&Expression> for GenericType {
    fn from(value: &Expression) -> GenericType {
        let lit = match value {
            Expression::Name(id) => id.clone(),
            _ => String::new()
        };

        GenericType::Single { lit, generics: vec![] }
    }
}
