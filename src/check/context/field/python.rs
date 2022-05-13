use std::collections::HashSet;

use python_parser::ast::{Expression, SetItem};

use crate::check::context::field::generic::{GenericField, GenericFields};

impl From<(&Vec<Expression>, &Vec<Vec<Expression>>)> for GenericFields {
    fn from((ids, values): (&Vec<Expression>, &Vec<Vec<Expression>>)) -> GenericFields {
        GenericFields {
            fields: ids
                .iter()
                .zip(values)
                .flat_map(|(id, values)| GenericFields::from((id, values)).fields)
                .collect()
        }
    }
}

impl From<(&Vec<Expression>, &Vec<Expression>)> for GenericFields {
    fn from((ids, values): (&Vec<Expression>, &Vec<Expression>)) -> GenericFields {
        GenericFields {
            fields: ids
                .iter()
                .zip(values)
                .flat_map(|(id, _)| GenericFields::from(id).fields)
                .collect()
        }
    }
}

impl From<(&Expression, &Vec<Expression>)> for GenericFields {
    // TODO infer type from values
    fn from((id, _): (&Expression, &Vec<Expression>)) -> GenericFields { GenericFields::from(id) }
}

impl From<&Expression> for GenericFields {
    fn from(id: &Expression) -> GenericFields {
        GenericFields {
            fields: (match id {
                Expression::Name(name) => vec![GenericField {
                    is_py_type: true,
                    name:       name.clone(),
                    pos:        Default::default(),
                    mutable:    false,
                    in_class:   None,
                    ty:         None
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
                                name:       name.clone(),
                                pos:        Default::default(),
                                mutable:    false,
                                in_class:   None,
                                ty:         None
                            },
                            _ => unreachable!()
                        }
                    })
                    .collect(),
                _ => vec![]
            })
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
        }
    }
}
