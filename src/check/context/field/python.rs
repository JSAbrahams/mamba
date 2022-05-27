use std::collections::HashSet;

use python_parser::ast::{Expression, SetItem};

use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::name::Name;

impl From<(&Vec<Expression>, &Option<Expression>)> for GenericFields {
    fn from((ids, ty): (&Vec<Expression>, &Option<Expression>)) -> GenericFields {
        let fields = GenericFields {
            fields: ids
                .iter()
                .flat_map(|id| GenericFields::from(id).fields)
                .collect()
        };

        if fields.fields.len() > 1 {
            fields // cannot type annotate tuples in python
        } else if let Some(ty) = ty {
            let name = Name::from(ty);
            if let Some(field) = fields.fields.iter().next() {
                let field = field.with_ty(&name);
                GenericFields { fields: HashSet::from([field]) }
            } else {
                fields
            }
        } else {
            fields
        }
    }
}

impl From<(&Expression, &Option<Expression>)> for GenericFields {
    fn from((id, _): (&Expression, &Option<Expression>)) -> GenericFields { GenericFields::from(id) }
}

impl From<&Expression> for GenericFields {
    fn from(id: &Expression) -> GenericFields {
        GenericFields {
            fields: (match id {
                Expression::Name(name) => vec![GenericField {
                    is_py_type: true,
                    name: name.clone(),
                    pos: Default::default(),
                    mutable: true,
                    in_class: None,
                    ty: None,
                    assigned_to: false, // unknown
                }],
                Expression::TupleLiteral(items) => items
                    .iter()
                    .filter(|item| matches!(item, SetItem::Unique(_)))
                    .filter(|item| match &item {
                        SetItem::Star(_) => false,
                        SetItem::Unique(expr) => matches!(expr, Expression::Name(_))
                    })
                    .map(|item| match &item {
                        SetItem::Star(_) => unreachable!(),
                        SetItem::Unique(expression) => match expression {
                            Expression::Name(name) => GenericField {
                                is_py_type: true,
                                name: name.clone(),
                                pos: Default::default(),
                                mutable: false,
                                in_class: None,
                                ty: None,
                                assigned_to: false, // unknown
                            },
                            _ => unreachable!()
                        }
                    })
                    .collect(),
                _ => vec![]
            }).iter().cloned().collect::<HashSet<_>>()
        }
    }
}
