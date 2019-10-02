use crate::type_checker::context::type_name::generic::GenericTypeName;
use python_parser::ast::Expression;

pub const INTEGER: &'static str = "int";
pub const FLOAT: &'static str = "float";
pub const STRING: &'static str = "str";
pub const BOOLEAN: &'static str = "bool";

impl From<&Expression> for GenericTypeName {
    fn from(value: &Expression) -> GenericTypeName {
        GenericTypeName::from(
            match value {
                Expression::Name(id) => id.clone(),
                _ => String::new()
            }
            .as_str()
        )
    }
}
