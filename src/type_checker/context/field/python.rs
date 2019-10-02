use crate::type_checker::context::field::generic::GenericField;
use python_parser::ast::{Expression, SetItem};

pub struct GenericFields {
    pub fields: Vec<GenericField>
}

impl From<(&Vec<Expression>, &Vec<Vec<Expression>>)> for GenericFields {
    fn from((ids, values): (&Vec<Expression>, &Vec<Vec<Expression>>)) -> GenericFields {
        GenericFields {
            fields: ids
                .iter()
                .zip(values)
                .map(|(id, values)| GenericFields::from((id, values)).fields)
                .flatten()
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
                .map(|(id, _)| GenericFields::from(id).fields)
                .flatten()
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
            fields: match id {
                Expression::Name(name) => vec![GenericField {
                    is_py_type: true,
                    name:       name.clone(),
                    pos:        Default::default(),
                    private:    false,
                    mutable:    false,
                    ty:         None
                }],
                Expression::TupleLiteral(items) => items
                    .iter()
                    .filter(|item| if let SetItem::Unique(_) = item { true } else { false })
                    .filter(|item| match &item {
                        SetItem::Star(_) => false,
                        SetItem::Unique(expression) =>
                            if let Expression::Name(_) = expression {
                                true
                            } else {
                                false
                            },
                    })
                    .map(|item| match &item {
                        SetItem::Star(_) => unreachable!(),
                        SetItem::Unique(expression) => match expression {
                            Expression::Name(name) => GenericField {
                                is_py_type: true,
                                name:       name.clone(),
                                pos:        Default::default(),
                                private:    false,
                                mutable:    false,
                                ty:         None
                            },
                            _ => unreachable!()
                        }
                    })
                    .collect(),
                _ => vec![]
            }
        }
    }
}
