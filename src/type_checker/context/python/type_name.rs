use python_parser::ast::Expression;

use crate::type_checker::context::generic::type_name::GenericTypeName;

pub const INTEGER: &'static str = "Integer";
pub const FLOAT: &'static str = "Float";
pub const STRING: &'static str = "String";
pub const BOOLEAN: &'static str = "Boolean";

impl From<&Expression> for GenericTypeName {
    fn from(value: &Expression) -> GenericTypeName {
        let lit = match value {
            Expression::Name(id) => id.clone(),
            _ => String::new()
        };

        GenericTypeName::Single { lit, generics: vec![] }
    }
}
