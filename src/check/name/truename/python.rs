use std::ops::Deref;

use python_parser::ast::{Expression, SetItem, Subscript};

use crate::check::context::clss::python::python_to_concrete;
use crate::check::name::nameunion::NameUnion;
use crate::check::name::namevariant::NameVariant;
use crate::check::name::truename::TrueName;

impl From<&Expression> for TrueName {
    fn from(value: &Expression) -> TrueName {
        match value {
            Expression::Name(id) => TrueName::from(python_to_concrete(&id.clone()).as_str()),
            Expression::TupleLiteral(items) => {
                let expressions = items.iter().filter_map(|setitem| match setitem {
                    SetItem::Star(_) => None,
                    SetItem::Unique(expr) => Some(expr)
                });
                let variant = NameVariant::Tuple(expressions.map(NameUnion::from).collect());
                TrueName::from(&variant)
            }
            Expression::Subscript(id, exprs) => {
                let lit = match &id.deref() {
                    Expression::Name(name) => name.clone(),
                    _ => return TrueName::empty()
                };

                // Union not expected
                if &lit == "Union" {
                    TrueName::empty()
                } else {
                    let generics: Vec<_> = exprs.iter().map(to_ty_name).collect();
                    let generics: Vec<NameUnion> = generics.iter().map(NameUnion::from).collect();
                    TrueName::new(&python_to_concrete(&lit), &generics)
                }
            }
            _ => TrueName::empty()
        }
    }
}

pub fn to_ty_name(sub_script: &Subscript) -> TrueName {
    match sub_script {
        Subscript::Simple(expr) => TrueName::from(expr),
        _ => TrueName::empty()
    }
}
