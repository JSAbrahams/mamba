use std::ops::Deref;

use python_parser::ast::Subscript::Simple;
use python_parser::ast::{Argument, Expression, Subscript};

use crate::check::context::parameter::generic::GenericParameter;
use crate::check::context::ty::python::python_to_concrete;

pub struct GenericParameters {
    pub parameters: Vec<GenericParameter>
}

// TODO add check that Python file does indeed import generic from typing
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
                            name:       python_to_concrete(&name.clone()),
                            parent:     None
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
            if let Subscript::Simple(expr) = subscript {
                if let Expression::Name(name) = expr {
                    parameters.push(GenericParameter {
                        is_py_type: true,
                        name:       python_to_concrete(&name.clone()),
                        parent:     None
                    })
                }
            }
        });

        GenericParameters { parameters }
    }
}
