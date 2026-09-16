use ruff_python_ast::{Arguments, Expr};

use crate::check::context::clss::python::python_to_concrete;
use crate::check::context::parameter::generic::GenericParameter;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::python::subscript_elements;

pub struct GenericParameters {
    pub parameters: Vec<GenericParameter>,
}

impl From<&Arguments> for GenericParameters {
    fn from(arguments: &Arguments) -> Self {
        let mut parameters = vec![];
        for argument in &arguments.args {
            let subscript = match argument {
                Expr::Subscript(subscript) if matches!(subscript.value.as_ref(), Expr::Name(name) if name.id.as_str() == "Generic") => {
                    subscript
                }
                _ => continue,
            };

            for name in subscript_elements(&subscript.slice) {
                if let Expr::Name(name) = name {
                    parameters.push(GenericParameter {
                        is_py_type: true,
                        name: StringName::from(python_to_concrete(name.id.as_str()).as_str()),
                        parent: None,
                    })
                }
            }
        }

        GenericParameters { parameters }
    }
}
