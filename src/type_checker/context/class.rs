use std::convert::TryFrom;
use std::ops::Deref;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::context::function_arg::FunctionArg;
use crate::type_checker::type_result::{TypeErr, TypeResult};

// TODO differentiate between class and type, a class has generic parameters, a
// type has generics
#[derive(Debug, Clone)]
pub struct Type {
    pub name:      String,
    pub args:      Vec<FunctionArg>,
    pub generics:  Vec<GenericParameter>,
    pub concrete:  bool,
    pub fields:    Vec<Field>,
    pub functions: Vec<Function>,
    pub parents:   Vec<Parent>
}

#[derive(Debug, Clone)]
pub struct GenericParameter {
    pub name:   String,
    pub parent: Option<String>
}

#[derive(Debug, Clone)]
pub struct Parent {
    pub name:     String,
    pub generics: Vec<GenericParameter>
}

impl Type {
    pub fn all_pure(self, all_pure: bool) -> Type {
        Type {
            name:      self.name,
            args:      self.args,
            generics:  self.generics,
            concrete:  self.concrete,
            fields:    self.fields,
            functions: self.functions.into_iter().map(|function| function.pure(all_pure)).collect(),
            parents:   self.parents
        }
    }

    pub fn overrides_op(&self, op: &Node) -> bool { unimplemented!() }
}

impl TryFrom<&AST> for Type {
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
                        let argument = FunctionArg::try_from(arg)?;
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
                let args: Vec<_> = args.into_iter().map(Result::unwrap).collect();

                let (fields, functions) = get_fields_and_functions(statements)?;

                let (parents, parent_errs): (Vec<_>, Vec<_>) =
                    parents.iter().map(Parent::try_from).partition(Result::is_ok);
                if !parent_errs.is_empty() {
                    return Err(parent_errs.into_iter().map(Result::unwrap_err).collect());
                }
                let parents = parents.into_iter().map(Result::unwrap).collect::<Vec<Parent>>();

                Ok(Type { name, args, generics, concrete: true, fields, functions, parents })
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

                let (fields, functions) = get_fields_and_functions(&statements)?;
                // TODO add parents to type definitions
                let parents = vec![];
                let args = vec![];
                Ok(Type { name, args, concrete: false, generics, fields, functions, parents })
            }
            _ => Err(vec![TypeErr::new(&class.pos, "Expected class or type definition")])
        }
    }
}

impl TryFrom<&AST> for GenericParameter {
    type Error = TypeErr;

    fn try_from(generic: &AST) -> Result<Self, Self::Error> {
        match &generic.node {
            Node::Generic { id, isa } => Ok(GenericParameter {
                name:   try_from_id(id)?,
                parent: match isa {
                    Some(isa) => Some(try_from_id(isa)?),
                    None => None
                }
            }),
            _ => Err(TypeErr::new(&generic.pos, "Expected generic"))
        }
    }
}

impl TryFrom<&AST> for Parent {
    type Error = TypeErr;

    fn try_from(generic: &AST) -> Result<Self, Self::Error> {
        match &generic.node {
            Node::Parent { id, generics, .. } => Ok(Parent {
                name:     try_from_id(id)?,
                generics: generics
                    .iter()
                    .map(GenericParameter::try_from)
                    .collect::<Result<Vec<GenericParameter>, TypeErr>>()?
            }),
            _ => Err(TypeErr::new(&generic.pos, "Expected generic"))
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

            Ok((
                try_from_id(id.deref()).map_err(|err| vec![err])?,
                generics.into_iter().map(Result::unwrap).collect::<Vec<GenericParameter>>()
            ))
        }
        _ => Err(vec![TypeErr::new(&_type.pos, "Expected class name")])
    }
}

fn get_fields_and_functions(
    statements: &[AST]
) -> Result<(Vec<Field>, Vec<Function>), Vec<TypeErr>> {
    let mut field_res = vec![];
    let mut fun_res = vec![];
    let mut errs = vec![];
    statements.iter().for_each(|statement| match &statement.node {
        Node::FunDef { .. } =>
            fun_res.push(Function::try_from(statement).and_then(|f| f.in_class(true))),
        Node::VariableDef { .. } => field_res.push(Field::try_from(statement)),
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
