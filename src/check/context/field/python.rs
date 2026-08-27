use std::collections::HashSet;

use python_parser::ast::{Expression, SetItem};

use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::name::Name;
use crate::common::position::Position;

impl From<(&Vec<Expression>, &Option<Expression>)> for GenericFields {
    fn from((ids, ty): (&Vec<Expression>, &Option<Expression>)) -> GenericFields {
        let fields = GenericFields {
            fields: ids
                .iter()
                .flat_map(|id| GenericFields::from(id).fields)
                .collect(),
        };

        if let Some(ty) = ty {
            let name = Name::from(ty);
            if let Some(field) = fields.fields.iter().next() {
                let field = field.with_ty(&name); // cannot annotate tuples in python
                GenericFields {
                    fields: HashSet::from([field]),
                }
            } else {
                fields
            }
        } else {
            fields
        }
    }
}

impl From<(&Expression, &Option<Expression>)> for GenericFields {
    fn from((id, _): (&Expression, &Option<Expression>)) -> GenericFields {
        GenericFields::from(id)
    }
}

impl From<&Expression> for GenericFields {
    fn from(id: &Expression) -> GenericFields {
        GenericFields {
            fields: (match id {
                Expression::Name(name) => vec![GenericField {
                    is_py_type: true,
                    name: name.clone(),
                    pos: Position::invisible(),
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
                        SetItem::Unique(expr) => matches!(expr, Expression::Name(_)),
                    })
                    .map(|item| match &item {
                        SetItem::Star(_) => unreachable!(),
                        SetItem::Unique(expression) => match expression {
                            Expression::Name(name) => GenericField {
                                is_py_type: true,
                                name: name.clone(),
                                pos: Position::invisible(),
                                mutable: false,
                                in_class: None,
                                ty: None,
                                assigned_to: false, // unknown
                            },
                            _ => unreachable!(),
                        },
                    })
                    .collect(),
                _ => vec![],
            })
            .iter()
            .cloned()
            .collect::<HashSet<_>>(),
        }
    }
}

#[cfg(test)]
mod test {
    use python_parser::ast::Statement;

    use crate::check::context::field::generic::GenericFields;

    fn assignment_targets(source: &str) -> (Vec<python_parser::ast::Expression>, bool) {
        let (_, statements) =
            python_parser::file_input(python_parser::make_strspan(source)).expect("parse source");
        match statements.first().expect("non empty statements") {
            Statement::Assignment(left, _) => (left.clone(), false),
            Statement::TypedAssignment(left, _, _) => (left.clone(), true),
            other => panic!("Not an assignment but {other:?}"),
        }
    }

    #[test]
    fn single_name() {
        let (left, _) = assignment_targets("x = 0");
        let fields = GenericFields::from((&left, &None)).fields;

        assert_eq!(fields.len(), 1);
        let field = fields.iter().next().expect("field");
        assert_eq!(field.name, String::from("x"));
        assert!(field.mutable);
        assert!(field.ty.is_none());
    }

    #[test]
    fn typed_single_name() {
        let (left, _) = assignment_targets("x: int = 0");
        let fields = GenericFields::from((
            &left,
            &Some(python_parser::ast::Expression::Name(String::from("int"))),
        ))
        .fields;

        assert_eq!(fields.len(), 1);
        let field = fields.iter().next().expect("field");
        assert_eq!(field.name, String::from("x"));
        assert!(field.ty.is_some());
    }

    #[test]
    fn tuple_destructuring() {
        // A single *parenthesized* tuple target (`(a, b) = 0, 0`) parses as one
        // `Expression::TupleLiteral`, unlike the unparenthesized `a, b = 0, 0` (which the parser
        // treats as two separate assignment targets, each going through the `Expression::Name`
        // branch instead). A tuple target cannot be annotated in Python either way, so this
        // always goes through the untyped `From<(&Vec<Expression>, &Option<Expression>)>` path.
        let (left, _) = assignment_targets("(a, b) = 0, 0");
        let fields = GenericFields::from((&left, &None)).fields;

        assert_eq!(fields.len(), 2);
        let mut names: Vec<&String> = fields.iter().map(|f| &f.name).collect();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
        assert!(fields.iter().all(|f| !f.mutable));
        assert!(fields.iter().all(|f| f.ty.is_none()));
    }
}
