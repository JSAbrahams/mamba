use ruff_python_ast::Expr;

use crate::check::context::clss::python::{python_to_concrete, UNION};
use crate::check::name::true_name::TrueName;
use crate::check::name::{Empty, Name, TupleCallable};

impl From<&Expr> for TrueName {
    fn from(value: &Expr) -> TrueName {
        match value {
            Expr::Name(name) => TrueName::from(python_to_concrete(name.id.as_str()).as_str()),
            Expr::Tuple(tuple) => {
                let expressions = tuple
                    .elts
                    .iter()
                    .filter(|element| !matches!(element, Expr::Starred(_)));
                TrueName::tuple(expressions.map(Name::from).collect::<Vec<_>>().as_slice())
            }
            Expr::Subscript(subscript) => {
                let lit = match subscript.value.as_ref() {
                    Expr::Name(name) => name.id.as_str(),
                    _ => return TrueName::empty(),
                };

                // Union not expected
                if lit == UNION {
                    TrueName::empty()
                } else {
                    let generics: Vec<_> = subscript_elements(&subscript.slice)
                        .map(TrueName::from)
                        .collect();
                    let generics: Vec<Name> = generics.iter().map(Name::from).collect();
                    TrueName::new(&python_to_concrete(lit), &generics)
                }
            }
            _ => TrueName::empty(),
        }
    }
}

/// The individual types a subscript is applied to.
///
/// Python's AST has one slice expression per subscript, so `X[a, b]` is a subscript of a single
/// tuple, while `X[a]` is a subscript of a bare `a`. Both are one generic argument per element
/// here.
pub fn subscript_elements(slice: &Expr) -> impl Iterator<Item = &Expr> {
    match slice {
        Expr::Tuple(tuple) => tuple.elts.iter().collect::<Vec<_>>(),
        other => vec![other],
    }
    .into_iter()
}
