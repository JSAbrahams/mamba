use std::ops::Deref;

use python_parser::ast::{Expression, SetItem, Subscript};

use crate::check::context::clss::python::python_to_concrete;
use crate::check::context::name::{Name, NameUnion, NameVariant};

impl From<&Expression> for NameUnion {
    fn from(value: &Expression) -> Self {
        match value {
            Expression::Name(_) => NameUnion::from(&Name::from(value)),
            Expression::TupleLiteral(_) => NameUnion::from(&Name::from(value)),
            Expression::Subscript(id, exprs) =>
                if id.deref() == &Expression::Name(String::from("Union")) {
                    let names: Vec<Name> = exprs.iter().map(|e| to_ty_name(e)).collect();
                    NameUnion::new(&names)
                } else {
                    NameUnion::from(&Name::from(value))
                },
            _ => NameUnion::from(&Name::empty())
        }
    }
}

impl From<&Expression> for Name {
    fn from(value: &Expression) -> Name {
        match value {
            Expression::Name(id) => Name::from(python_to_concrete(&id.clone()).as_str()),
            Expression::TupleLiteral(items) => {
                let expressions = items.iter().filter_map(|setitem| match setitem {
                    SetItem::Star(_) => None,
                    SetItem::Unique(expr) => Some(expr)
                });
                let variant = NameVariant::Tuple(expressions.map(NameUnion::from).collect());
                Name::from(&variant)
            }
            Expression::Subscript(id, exprs) => {
                let lit = match &id.deref() {
                    Expression::Name(name) => name.clone(),
                    _ => return Name::empty()
                };

                // Union not expected
                if &lit == "Union" {
                    Name::empty()
                } else {
                    let generics: Vec<_> = exprs.iter().map(|e| to_ty_name(e)).collect();
                    let generics: Vec<NameUnion> = generics.iter().map(NameUnion::from).collect();
                    Name::new(&python_to_concrete(&lit), &generics)
                }
            }
            _ => Name::empty()
        }
    }
}

fn to_ty_name(sub_script: &Subscript) -> Name {
    match sub_script {
        Subscript::Simple(expr) => Name::from(expr),
        _ => Name::empty()
    }
}
