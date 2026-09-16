use std::collections::HashSet;

use ruff_python_ast::Expr;

use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::name::Name;
use crate::common::position::Position;

impl From<(&[Expr], Option<&Expr>)> for GenericFields {
    fn from((ids, ty): (&[Expr], Option<&Expr>)) -> GenericFields {
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

impl From<&Expr> for GenericFields {
    fn from(id: &Expr) -> GenericFields {
        GenericFields {
            fields: match id {
                Expr::Name(name) => HashSet::from([GenericField {
                    is_py_type: true,
                    name: name.id.as_str().to_string(),
                    pos: Position::invisible(),
                    mutable: true,
                    in_class: None,
                    ty: None,
                }]),
                Expr::Tuple(tuple) => tuple
                    .elts
                    .iter()
                    .filter_map(|element| match element {
                        Expr::Name(name) => Some(GenericField {
                            is_py_type: true,
                            name: name.id.as_str().to_string(),
                            pos: Position::invisible(),
                            mutable: false,
                            in_class: None,
                            ty: None,
                        }),
                        _ => None,
                    })
                    .collect(),
                _ => HashSet::new(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use ruff_python_ast::{Expr, Stmt};

    use crate::check::context::field::generic::GenericFields;
    use crate::check::context::python::python_stmts;

    fn assignment_targets(source: &str) -> (Vec<Expr>, Option<Expr>) {
        let statements = python_stmts(source).expect("parse source");
        match statements.first().expect("non empty statements") {
            Stmt::Assign(assign) => (assign.targets.clone(), None),
            Stmt::AnnAssign(assign) => (
                vec![assign.target.as_ref().clone()],
                Some(assign.annotation.as_ref().clone()),
            ),
            other => panic!("Not an assignment but {other:?}"),
        }
    }

    #[test]
    fn single_name() {
        let (left, _) = assignment_targets("x = 0");
        let fields = GenericFields::from((left.as_slice(), None)).fields;

        assert_eq!(fields.len(), 1);
        let field = fields.iter().next().expect("field");
        assert_eq!(field.name, String::from("x"));
        assert!(field.mutable);
        assert!(field.ty.is_none());
    }

    #[test]
    fn typed_single_name() {
        let (left, ty) = assignment_targets("x: int = 0");
        let fields = GenericFields::from((left.as_slice(), ty.as_ref())).fields;

        assert_eq!(fields.len(), 1);
        let field = fields.iter().next().expect("field");
        assert_eq!(field.name, String::from("x"));
        assert!(field.ty.is_some());
    }

    #[test]
    fn tuple_destructuring() {
        // A tuple target is one `Expr::Tuple`, parenthesized or not, so `a, b = 0, 0` and
        // `(a, b) = 0, 0` both go through the `Expr::Tuple` branch rather than the
        // `Expr::Name` one. A tuple target cannot be annotated in Python either way, so this
        // always goes through the untyped path.
        let (left, _) = assignment_targets("(a, b) = 0, 0");
        let fields = GenericFields::from((left.as_slice(), None)).fields;

        assert_eq!(fields.len(), 2);
        let mut names: Vec<&String> = fields.iter().map(|f| &f.name).collect();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
        assert!(fields.iter().all(|f| !f.mutable));
        assert!(fields.iter().all(|f| f.ty.is_none()));
    }
}
