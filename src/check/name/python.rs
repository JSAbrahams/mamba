use ruff_python_ast::Expr;

use crate::check::context::clss::python::UNION;
use crate::check::name::true_name::python::subscript_elements;
use crate::check::name::true_name::TrueName;
use crate::check::name::{Empty, Name};

impl From<&Expr> for Name {
    fn from(value: &Expr) -> Self {
        match value {
            Expr::Name(_) | Expr::Tuple(_) => Name::from(&TrueName::from(value)),
            Expr::Subscript(subscript) => {
                let is_union = matches!(subscript.value.as_ref(), Expr::Name(name) if name.id.as_str() == UNION);
                if is_union {
                    let names: Vec<TrueName> = subscript_elements(&subscript.slice)
                        .map(TrueName::from)
                        .collect();
                    Name::from(&names)
                } else {
                    Name::from(&TrueName::from(value))
                }
            }
            _ => Name::from(&TrueName::empty()),
        }
    }
}
