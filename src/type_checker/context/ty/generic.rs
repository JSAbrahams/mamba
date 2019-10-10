use std::convert::TryFrom;
use std::hash::{Hash, Hasher};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::context::field::generic::GenericField;
use crate::type_checker::context::function::concrete;
use crate::type_checker::context::function::generic::GenericFunction;
use crate::type_checker::context::function_arg::generic::{ClassArgument, GenericFunctionArg};
use crate::type_checker::context::parameter::GenericParameter;
use crate::type_checker::context::parent::generic::GenericParent;
use crate::type_checker::context::type_name::actual::ActualTypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Debug, Clone, Eq)]
pub struct GenericType {
    pub is_py_type: bool,
    pub name:       ActualTypeName,
    pub pos:        Position,
    pub concrete:   bool,
    pub args:       Vec<GenericFunctionArg>,
    pub generics:   Vec<GenericParameter>,
    pub fields:     HashSet<GenericField>,
    pub functions:  HashSet<GenericFunction>,
    pub parents:    HashSet<GenericParent>
}

impl PartialEq for GenericType {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl Hash for GenericType {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl GenericType {
    pub fn all_pure(self, pure: bool) -> Result<Self, TypeErr> {
        Ok(GenericType {
            functions: self.functions.iter().map(|f| f.clone().pure(pure)).collect(),
            ..self
        })
    }
}

impl TryFrom<&AST> for GenericType {
    type Error = Vec<TypeErr>;

    fn try_from(class: &AST) -> TypeResult<GenericType> {
        match &class.node {
            // TODO add pure classes
            Node::Class { _type, args, parents, body } => {
                let (name, generics) = get_name_and_generics(_type)?;
                let statements = match &body.node {
                    Node::Block { statements } => statements,
                    _ => return Err(vec![TypeErr::new(&class.pos, "Expected block in class")])
                };

                let mut class_args = vec![];
                let mut arg_errs = vec![];
                let mut argument_fields = HashSet::new();
                args.iter().for_each(|arg| match ClassArgument::try_from(arg) {
                    Err(err) => arg_errs.push(err),
                    Ok(ClassArgument { field: None, fun_arg }) => {
                        class_args.push(fun_arg);
                    }
                    Ok(ClassArgument { field: Some(field), fun_arg }) => {
                        class_args.push(fun_arg);
                        argument_fields.insert(field);
                    }
                });
                if !arg_errs.is_empty() {
                    return Err(arg_errs.into_iter().flatten().collect());
                }

                let (body_fields, functions) = get_fields_and_functions(&name, statements)?;
                for function in functions.clone() {
                    if function.name == ActualTypeName::new(concrete::INIT, &vec![]) {
                        if class_args.is_empty() {
                            class_args.append(&mut function.arguments.clone())
                        } else {
                            return Err(vec![TypeErr::new(
                                &class.pos,
                                "Cannot have constructor and class arguments"
                            )]);
                        }
                    }
                }

                let (parents, parent_errs): (Vec<_>, Vec<_>) =
                    parents.iter().map(GenericParent::try_from).partition(Result::is_ok);
                if !parent_errs.is_empty() {
                    return Err(parent_errs
                        .into_iter()
                        .map(Result::unwrap_err)
                        .flatten()
                        .collect());
                }

                Ok(GenericType {
                    is_py_type: false,
                    name,
                    pos: class.pos.clone(),
                    args: class_args,
                    generics,
                    concrete: true,
                    fields: argument_fields.union(&body_fields).cloned().collect(),
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

                let (fields, functions) = get_fields_and_functions(&name, &statements)?;
                // TODO add parents to type definitions
                Ok(GenericType {
                    is_py_type: false,
                    name,
                    pos: class.pos.clone(),
                    args: vec![],
                    concrete: false,
                    generics,
                    fields,
                    functions,
                    parents: HashSet::new()
                })
            }
            _ => Err(vec![TypeErr::new(&class.pos, "Expected class or type definition")])
        }
    }
}

fn get_name_and_generics(
    _type: &AST
) -> Result<(ActualTypeName, Vec<GenericParameter>), Vec<TypeErr>> {
    match &_type.node {
        Node::Type { id, generics } => {
            let (generics, generic_errs): (Vec<_>, Vec<_>) =
                generics.iter().map(GenericParameter::try_from).partition(Result::is_ok);
            if !generic_errs.is_empty() {
                return Err(generic_errs.into_iter().map(Result::unwrap_err).flatten().collect());
            }

            let name = ActualTypeName::new(
                match &id.node {
                    Node::Id { lit } => lit.clone(),
                    _ => return Err(vec![TypeErr::new(&id.pos, "Expected identifier")])
                }
                .as_str(),
                &vec![]
            );

            Ok((name, generics.into_iter().map(Result::unwrap).collect::<Vec<_>>()))
        }
        _ => Err(vec![TypeErr::new(&_type.pos, "Expected class name")])
    }
}

fn get_fields_and_functions(
    class: &ActualTypeName,
    statements: &[AST]
) -> Result<(HashSet<GenericField>, HashSet<GenericFunction>), Vec<TypeErr>> {
    let mut field_res = vec![];
    let mut fun_res = vec![];
    let mut errs = vec![];
    statements.iter().for_each(|statement| match &statement.node {
        Node::FunDef { .. } => {
            let function =
                GenericFunction::try_from(statement).and_then(|f| f.in_class(Some(class)));

            fun_res.push(function);
        }
        Node::VariableDef { .. } => field_res.push(GenericField::try_from(statement)),
        Node::Comment { .. } => {}
        _ => errs.push(TypeErr::new(&statement.pos, "Expected function or variable definition"))
    });

    let (fields, field_errs): (Vec<_>, Vec<_>) = field_res.into_iter().partition(Result::is_ok);
    let (functions, function_errs): (Vec<_>, Vec<_>) = fun_res.into_iter().partition(Result::is_ok);

    // TODO check that there are no duplicate fields or functions

    if !field_errs.is_empty() || !function_errs.is_empty() || !errs.is_empty() {
        errs.append(&mut field_errs.into_iter().map(Result::unwrap_err).flatten().collect());
        errs.append(&mut function_errs.into_iter().map(Result::unwrap_err).flatten().collect());
        Err(errs)
    } else {
        Ok((
            HashSet::from_iter(fields.into_iter().map(Result::unwrap)),
            HashSet::from_iter(functions.into_iter().map(Result::unwrap))
        ))
    }
}
