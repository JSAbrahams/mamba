use crate::type_checker::context::parameter::generic::GenericParameter;
use python_parser::ast::Subscript::Simple;
use python_parser::ast::{Argument, Expression};
use std::ops::Deref;

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
                            name:       name.clone(),
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
