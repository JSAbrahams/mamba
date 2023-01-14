use std::ops::Deref;

use python_parser::ast::Expression;

use crate::check::context::clss::python::UNION;
use crate::check::name::{Empty, Name};
use crate::check::name::true_name::python::to_ty_name;
use crate::check::name::true_name::TrueName;

impl From<&Expression> for Name {
    fn from(value: &Expression) -> Self {
        match value {
            Expression::Name(_) | Expression::TupleLiteral(_) => Name::from(&TrueName::from(value)),
            Expression::Subscript(id, exprs) =>
                if id.deref() == &Expression::Name(String::from(UNION)) {
                    let names: Vec<TrueName> = exprs.iter().map(to_ty_name).collect();
                    Name::from(&names)
                } else {
                    Name::from(&TrueName::from(value))
                },
            _ => Name::from(&TrueName::empty())
        }
    }
}
