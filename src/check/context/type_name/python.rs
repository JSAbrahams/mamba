use crate::check::context::ty::python::python_to_concrete;
use crate::check::ty_name::TypeName;
use python_parser::ast::{Expression, Subscript};
use std::ops::Deref;

pub const INTEGER: &str = "int";
pub const FLOAT: &str = "float";
pub const STRING: &str = "str";
pub const BOOLEAN: &str = "bool";

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

                if lit == String::from("Union") {
                    let names: Vec<_> = exprs.iter().map(|e| to_ty_name(e)).collect();
                    if let Some(mut first) = names.first().cloned() {
                        for name in names {
                            first = first.union(&name).clone()
                        }
                        first
                    } else {
                        TypeName::from("")
                    }
                } else {
                    let generics: Vec<_> = exprs.iter().map(|e| to_ty_name(e)).collect();
                    TypeName::new(&lit, &generics)
                }
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
