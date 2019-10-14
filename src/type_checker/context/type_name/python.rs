use crate::type_checker::context::ty::python::python_to_concrete;
use crate::type_checker::context::type_name::TypeName;
use python_parser::ast::Expression;

pub const INTEGER: &'static str = "int";
pub const FLOAT: &'static str = "float";
pub const STRING: &'static str = "str";
pub const BOOLEAN: &'static str = "bool";

// TODO handle type unions
// TODO handle generics
impl From<&Expression> for TypeName {
    fn from(value: &Expression) -> TypeName {
        TypeName::from(
            match value {
                Expression::Name(id) => python_to_concrete(&id.clone()),
                _ => String::new()
            }
            .as_str()
        )
    }
}
