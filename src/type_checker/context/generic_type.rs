use std::convert::TryFrom;

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic_field::GenericField;
use crate::type_checker::context::generic_function::GenericFunction;
use crate::type_checker::context::generic_function_arg::GenericFunctionArg;
use crate::type_checker::context::generic_parameter::GenericParameter;
use crate::type_checker::context::generic_parent::GenericParent;
use crate::type_checker::context::generic_type_name::GenericTypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};

#[derive(Debug, Clone)]
pub struct GenericType {
    pub name:      String,
    pub pos:       Position,
    pub concrete:  bool,
    pub args:      Vec<GenericFunctionArg>,
    pub generics:  Vec<GenericParameter>,
    pub fields:    Vec<GenericField>,
    pub functions: Vec<GenericFunction>,
    pub parents:   Vec<GenericParent>
}

impl GenericType {
    pub fn all_pure(self, pure: bool) -> Result<Self, TypeErr> {
        Ok(GenericType {
            name:      self.name,
            pos:       self.pos,
            concrete:  self.concrete,
            args:      self.args,
            generics:  self.generics,
            fields:    self.fields,
            functions: self.functions.iter().map(|f| f.clone().pure(pure)).collect(),
            parents:   self.parents
        })
    }
}

impl TryFrom<&AST> for GenericType {
    type Error = Vec<TypeErr>;

    fn try_from(class: &AST) -> TypeResult {
        match &class.node {
            // TODO add pure classes
            Node::Class { _type, args, parents, body } => {
                let (name, generics) = get_name_and_generics(_type)?;
                let statements = match &body.node {
                    Node::Block { statements } => statements,
                    _ => return Err(vec![TypeErr::new(&class.pos, "Expected block in class")])
                };

                let (args, argument_errs): (Vec<_>, Vec<_>) = args
                    .iter()
                    .map(|arg| {
                        let argument = GenericFunctionArg::try_from(arg)?;
                        if argument.vararg {
                            Err(TypeErr::new(
                                &arg.pos,
                                "Vararg currently not supported for class arguments"
                            ))
                        } else {
                            Ok(argument)
                        }
                    })
                    .partition(Result::is_ok);

                if !argument_errs.is_empty() {
                    return Err(argument_errs.into_iter().map(Result::unwrap_err).collect());
                }

                let (fields, functions) = get_fields_and_functions(&name, &generics, statements)?;

                let (parents, parent_errs): (Vec<_>, Vec<_>) =
                    parents.iter().map(GenericParent::try_from).partition(Result::is_ok);
                if !parent_errs.is_empty() {
                    return Err(parent_errs.into_iter().map(Result::unwrap_err).collect());
                }

                Ok(GenericType {
                    name,
                    pos: class.pos.clone(),
                    args: args.into_iter().map(Result::unwrap).collect(),
                    generics,
                    concrete: true,
                    fields,
                    functions,
                    parents: parents.into_iter().map(Result::unwrap).collect()
                })
            }
            Node::TypeDef { _type, body } => {
                let (name, generics) = get_name_and_generics(_type)?;
                let statements = if let Some(body) = body {
                    match &body.node {
                        Node::Block { statements } => statements.clone(),
                        _ => return Err(vec![TypeErr::new(&class.pos, "Expected block in class")])
                    }
                } else {
                    vec![]
                };

                let (fields, functions) = get_fields_and_functions(&name, &generics, &statements)?;
                // TODO add parents to type definitions
                let parents = vec![];
                let args = vec![];
                Ok(GenericType {
                    name,
                    pos: class.pos.clone(),
                    args,
                    concrete: false,
                    generics,
                    fields,
                    functions,
                    parents
                })
            }
            _ => Err(vec![TypeErr::new(&class.pos, "Expected class or type definition")])
        }
    }
}

fn get_name_and_generics(_type: &AST) -> Result<(String, Vec<GenericParameter>), Vec<TypeErr>> {
    match &_type.node {
        Node::Type { id, generics } => {
            let (generics, generic_errs): (Vec<_>, Vec<_>) =
                generics.iter().map(GenericParameter::try_from).partition(Result::is_ok);
            if !generic_errs.is_empty() {
                return Err(generic_errs.into_iter().map(Result::unwrap_err).collect());
            }

            let name = match &id.node {
                Node::Id { lit } => Ok(lit.clone()),
                _ => Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
            }?;

            Ok((name, generics.into_iter().map(Result::unwrap).collect::<Vec<_>>()))
        }
        _ => Err(vec![TypeErr::new(&_type.pos, "Expected class name")])
    }
}

fn get_fields_and_functions(
    name: &String,
    generics: &Vec<GenericParameter>,
    statements: &[AST]
) -> Result<(Vec<GenericField>, Vec<GenericFunction>), Vec<TypeErr>> {
    let mut field_res = vec![];
    let mut fun_res = vec![];
    let mut errs = vec![];
    statements.iter().for_each(|statement| match &statement.node {
        Node::FunDef { .. } => {
            let function = GenericFunction::try_from(statement).and_then(|f| {
                f.in_class(Some(GenericTypeName::Single {
                    lit:      name.clone(),
                    generics: generics
                        .iter()
                        .map(|g| GenericTypeName::Single {
                            lit:      g.name.clone(),
                            generics: vec![]
                        })
                        .collect()
                }))
            });

            fun_res.push(function);
        }
        Node::VariableDef { .. } => field_res.push(GenericField::try_from(statement)),
        Node::Comment { .. } => {}
        _ => errs.push(TypeErr::new(&statement.pos, "Expected function or variable definition"))
    });

    let (fields, field_errs): (Vec<_>, Vec<_>) = field_res.into_iter().partition(Result::is_ok);
    let (functions, function_errs): (Vec<_>, Vec<_>) = fun_res.into_iter().partition(Result::is_ok);

    if !field_errs.is_empty() || !function_errs.is_empty() || !errs.is_empty() {
        errs.append(&mut field_errs.into_iter().map(Result::unwrap_err).collect());
        errs.append(&mut function_errs.into_iter().map(Result::unwrap_err).collect());
        Err(errs)
    } else {
        Ok((
            fields.into_iter().map(Result::unwrap).collect(),
            functions.into_iter().map(Result::unwrap).collect()
        ))
    }
}
