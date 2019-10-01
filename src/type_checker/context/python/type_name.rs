use python_parser::ast::Expression;

use crate::type_checker::context::generic::type_name::GenericActualTypeName;

pub const INTEGER: &'static str = "int";
pub const FLOAT: &'static str = "float";
pub const STRING: &'static str = "str";
pub const BOOLEAN: &'static str = "bool";

impl From<&Expression> for GenericActualTypeName {
    fn from(value: &Expression) -> GenericActualTypeName {
        let lit = match value {
            Expression::Name(id) => id.clone(),
            _ => String::new()
        };

        GenericActualTypeName::Single { lit, generics: vec![] }
    }
}
