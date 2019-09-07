use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::{Function, FunctionArg};
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Type {
    pub name:      String,
    pub args:      Vec<FunctionArg>,
    pub generics:  Vec<Generic>,
    pub concrete:  bool,
    pub fields:    Vec<Field>,
    pub functions: Vec<Function>,
    pub parents:   Vec<Parent>
}

#[derive(Debug, Clone)]
pub struct Generic {
    pub name:   String,
    pub parent: Option<String>
}

#[derive(Debug, Clone)]
pub struct Parent {
    pub name:     String,
    pub generics: Vec<Generic>
}

impl Type {
    pub fn try_from_node_pos(class: &ASTNodePos, all_pure: bool) -> TypeResult {
        match &class.node {
            // TODO add pure classes
            ASTNode::Class { _type, args, parents, body } => {
                let (name, generics) = get_name_and_generics(_type)?;
                let statements = match &body.node {
                    ASTNode::Block { statements } => statements,
                    _ => return Err(vec![TypeErr::new(class.position, "Expected block in class")])
                };

                let (args, argument_errs): (Vec<_>, Vec<_>) = args
                    .iter()
                    .map(|arg| {
                        let argument = FunctionArg::try_from_node_pos(arg)?;
                        if argument.vararg {
                            Err(TypeErr::new(
                                arg.position,
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

                let (fields, functions) = get_fields_and_functions(statements, all_pure)?;

                let (parents, parent_errs): (Vec<_>, Vec<_>) = parents
                    .iter()
                    .map(|parent| Parent::try_from_node_pos(parent))
                    .partition(Result::is_ok);
                if !parent_errs.is_empty() {
                    return Err(parent_errs.into_iter().map(Result::unwrap_err).collect());
                }
                let parents = parents.into_iter().map(Result::unwrap).collect::<Vec<Parent>>();

                Ok(Type { name, args, generics, concrete: true, fields, functions, parents })
            }
            ASTNode::TypeDef { _type, body } => {
                let (name, generics) = get_name_and_generics(_type)?;
                let statements = if let Some(body) = body {
                    match &body.node {
                        ASTNode::Block { statements } => statements.clone(),
                        _ =>
                            return Err(vec![TypeErr::new(
                                class.position,
                                "Expected block in class"
                            )]),
                    }
                } else {
                    vec![]
                };

                let (fields, functions) = get_fields_and_functions(&statements, all_pure)?;
                // TODO add parents to type definitions
                let parents = vec![];
                Ok(Type {
                    name,
                    args: vec![],
                    concrete: false,
                    generics,
                    fields,
                    functions,
                    parents
                })
            }
            _ => Err(vec![TypeErr::new(class.position, "Expected class or type definition")])
        }
    }
}

impl Generic {
    pub fn try_from_node_pos(generic: &ASTNodePos) -> Result<Generic, TypeErr> {
        match &generic.node {
            ASTNode::Generic { id, isa } => Ok(Generic {
                name:   try_from_id(id)?,
                parent: match isa {
                    Some(isa) => Some(try_from_id(isa)?),
                    None => None
                }
            }),
            _ => Err(TypeErr::new(generic.position, "Expected generic"))
        }
    }
}

impl Parent {
    pub fn try_from_node_pos(generic: &ASTNodePos) -> Result<Parent, TypeErr> {
        match &generic.node {
            ASTNode::Parent { id, generics, .. } => Ok(Parent {
                name:     try_from_id(id)?,
                generics: generics
                    .iter()
                    .map(|generic| Generic::try_from_node_pos(generic))
                    .collect::<Result<Vec<Generic>, TypeErr>>()?
            }),
            _ => Err(TypeErr::new(generic.position, "Expected generic"))
        }
    }
}

fn get_name_and_generics(_type: &ASTNodePos) -> Result<(String, Vec<Generic>), Vec<TypeErr>> {
    match &_type.node {
        ASTNode::Type { id, generics } => {
            let (generics, generic_errs): (Vec<_>, Vec<_>) = generics
                .iter()
                .map(|generic| Generic::try_from_node_pos(generic))
                .partition(Result::is_ok);
            if !generic_errs.is_empty() {
                return Err(generic_errs.into_iter().map(Result::unwrap_err).collect());
            }

            Ok((
                try_from_id(id.deref()).map_err(|err| vec![err])?,
                generics.into_iter().map(Result::unwrap).collect::<Vec<Generic>>()
            ))
        }
        _ => Err(vec![TypeErr::new(_type.position, "Expected class name")])
    }
}

fn get_fields_and_functions(
    statements: &[ASTNodePos],
    all_pure: bool
) -> Result<(Vec<Field>, Vec<Function>), Vec<TypeErr>> {
    let (mut field_res, mut fun_res, mut errs) = (vec![], vec![], vec![]);
    statements.iter().for_each(|statement| match &statement.node {
        ASTNode::FunDef { .. } => fun_res.push(Function::try_from_node_pos(statement, all_pure)),
        ASTNode::VariableDef { .. } => field_res.push(Field::try_from_node_pos(statement)),
        _ => errs.push(TypeErr::new(statement.position, "Expected function or variable definition"))
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
