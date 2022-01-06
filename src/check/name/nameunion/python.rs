use std::ops::Deref;

use python_parser::ast::Expression;

use crate::check::name::nameunion::NameUnion;
use crate::check::name::truename::python::to_ty_name;
use crate::check::name::truename::TrueName;

impl From<&Expression> for NameUnion {
    fn from(value: &Expression) -> Self {
        match value {
            Expression::Name(_) => NameUnion::from(&TrueName::from(value)),
            Expression::TupleLiteral(_) => NameUnion::from(&TrueName::from(value)),
            Expression::Subscript(id, exprs) =>
                if id.deref() == &Expression::Name(String::from("Union")) {
                    let names: Vec<TrueName> = exprs.iter().map(|e| to_ty_name(e)).collect();
                    NameUnion::new(&names)
                } else {
                    NameUnion::from(&TrueName::from(value))
                },
            _ => NameUnion::from(&TrueName::empty())
        }
    }
}
