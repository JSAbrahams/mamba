use crate::type_checker::context::ty::python::python_to_concrete;
use crate::type_checker::context::type_name::TypeName;
use python_parser::ast::{Expression, Subscript};
use std::ops::Deref;

pub const INTEGER: &'static str = "int";
pub const FLOAT: &'static str = "float";
pub const STRING: &'static str = "str";
pub const BOOLEAN: &'static str = "bool";

// TODO handle type unions
impl From<&Expression> for TypeName {
    fn from(value: &Expression) -> TypeName {
        match value {
            Expression::Name(id) => TypeName::from(python_to_concrete(&id.clone()).as_str()),
            Expression::Subscript(id, exprs) => {
                let lit = match &id.deref() {
                    Expression::Name(name) => name.clone(),
                    _ => String::new()
                };
                let generics: Vec<_> = exprs.iter().map(|e| to_ty_name(e)).collect();
                TypeName::new(&lit, &generics)
            }
            _ => TypeName::from("")
        }
    }
}

fn to_ty_name(sub_script: &Subscript) -> TypeName {
    match sub_script {
        Subscript::Simple(expr) => TypeName::from(expr),
        _ => TypeName::from("")
    }
}
