use std::ops::Deref;

use python_parser::ast::{Argument, Expression, Subscript};
use python_parser::ast::Subscript::Simple;

use crate::check::context::clss::python::python_to_concrete;
use crate::check::context::parameter::generic::GenericParameter;
use crate::check::name::string_name::StringName;

pub struct GenericParameters {
    pub parameters: Vec<GenericParameter>,
}

impl From<&Vec<Argument>> for GenericParameters {
    fn from(args: &Vec<Argument>) -> Self {
        let mut parameters = vec![];
        args.iter().for_each(|arg| match &arg {
            Argument::Positional(arg) => match &arg {
                Expression::Subscript(name, generics)
                if &Expression::Name(String::from("Generic")) == name.deref() =>
                    {
                        let name = generics.first();
                        if let Some(Simple(Expression::Name(name))) = name {
                            parameters.push(GenericParameter {
                                is_py_type: true,
                                name: StringName::from(python_to_concrete(name).as_str()),
                                parent: None,
                            })
                        }
                    }
                _ => {}
            },
            Argument::Starargs(_) => {}
            Argument::Keyword(..) => {}
            Argument::Kwargs(_) => {}
        });

        GenericParameters { parameters }
    }
}

impl From<&Vec<Subscript>> for GenericParameters {
    fn from(args: &Vec<Subscript>) -> Self {
        let mut parameters = vec![];
        args.iter().for_each(|subscript| {
            if let Subscript::Simple(Expression::Name(name)) = subscript {
                let name = StringName::from(python_to_concrete(name).as_str());
                parameters.push(GenericParameter { is_py_type: true, name, parent: None })
            }
        });

        GenericParameters { parameters }
    }
}
