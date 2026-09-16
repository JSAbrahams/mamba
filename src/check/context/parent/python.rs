use ruff_python_ast::Expr;

use crate::check::context::clss::python::python_to_concrete;
use crate::check::context::parent::generic::GenericParent;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::python::subscript_elements;
use crate::check::name::true_name::TrueName;
use crate::check::name::{Empty, Name};
use crate::common::position::Position;

impl From<&Expr> for GenericParent {
    fn from(expression: &Expr) -> GenericParent {
        let name = match expression {
            Expr::Name(name) => StringName::from(python_to_concrete(name.id.as_str()).as_str()),
            Expr::Subscript(subscript) => {
                if let Expr::Name(name) = subscript.value.as_ref() {
                    let generics: Vec<Name> = subscript_elements(&subscript.slice)
                        .map(Name::from)
                        .collect();
                    StringName::new(python_to_concrete(name.id.as_str()).as_ref(), &generics)
                } else {
                    StringName::empty()
                }
            }
            _ => StringName::empty(),
        };

        let name = TrueName::from(&name);
        GenericParent {
            is_py_type: true,
            name,
            pos: Position::invisible(),
        }
    }
}
