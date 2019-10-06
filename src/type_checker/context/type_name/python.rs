use crate::type_checker::context::type_name::TypeName;
use python_parser::ast::Expression;

pub const INTEGER: &'static str = "int";
pub const FLOAT: &'static str = "float";
pub const STRING: &'static str = "str";
pub const BOOLEAN: &'static str = "bool";

impl From<&Expression> for TypeName {
    fn from(value: &Expression) -> TypeName {
        TypeName::from(
            match value {
                Expression::Name(id) => id.clone(),
                _ => String::new()
            }
            .as_str()
        )
    }
}
