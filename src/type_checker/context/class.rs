use std::ops::Deref;

use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::context::common::try_from_id;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::{Function, FunctionArg};
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_node::Type;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub fn get_types(ast_tree: &ASTNodePos) -> TypeResult<Vec<Type>> {
    let modules = match &ast_tree.node {
        ASTNode::File { modules, .. } => modules,
        _ => panic!()
    };

    let (oks, errs): (Vec<_>, Vec<_>) = modules
        .iter()
        .map(|module| match &module.node {
            ASTNode::Class { .. } => get_class(module),
            ASTNode::TypeDef { .. } => get_type_def(module),
            _ => Err(vec![TypeErr::new(Position::from(module), "Expected either class or typedef")])
        })
        .partition(Result::is_ok);

    if errs.is_empty() {
        Ok(oks.into_iter().map(Result::unwrap).collect())
    } else {
        Err(errs.into_iter().flat_map(Result::unwrap_err).collect())
    }
}

fn get_class(class: &ASTNodePos) -> TypeResult {
    match &class.node {
        ASTNode::Class { _type, args, parents, body } => {
            let (name, generics) = get_name_and_generics(_type)?;
            let statements = match &body.node {
                ASTNode::Block { statements } => statements,
                _ =>
                    return Err(vec![TypeErr::new(
                        Position::from(class),
                        "Expected block in class"
                    )]),
            };

            let (args, argument_errs): (Vec<_>, Vec<_>) = args
                .iter()
                .map(|arg| {
                    let argument = FunctionArg::try_from_node_pos(arg)?;
                    if argument.vararg {
                        Err(TypeErr::new(
                            Position::from(arg),
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

            let (parents, parent_errs): (Vec<_>, Vec<_>) = parents
                .iter()
                .map(|parent| TypeName::try_from_node_pos(parent))
                .partition(Result::is_ok);
            if !parent_errs.is_empty() {
                return Err(parent_errs.into_iter().map(Result::unwrap_err).collect());
            }
            let parents = parents.into_iter().map(Result::unwrap).collect::<Vec<TypeName>>();

            Ok(Type { name, args, generics, concrete: true, fields, functions, parents })
        }
        _ => Err(vec![TypeErr::new(Position::from(class), "Expected class")])
    }
}

fn get_type_def(type_def: &ASTNodePos) -> TypeResult {
    match &type_def.node {
        ASTNode::TypeDef { _type, body } => {
            let (name, generics) = get_name_and_generics(_type)?;
            let statements = if let Some(body) = body {
                match &body.node {
                    ASTNode::Block { statements } => statements.clone(),
                    _ =>
                        return Err(vec![TypeErr::new(
                            Position::from(type_def),
                            "Expected block in class"
                        )]),
                }
            } else {
                vec![]
            };

            let (fields, functions) = get_fields_and_functions(&statements)?;
            // TODO add parents to type definitions
            let parents = vec![];
            Ok(Type { name, args: vec![], concrete: false, generics, fields, functions, parents })
        }
        _ => Err(vec![TypeErr::new(Position::from(type_def), "Expected type definition")])
    }
}

fn get_name_and_generics(_type: &ASTNodePos) -> Result<(String, Vec<TypeName>), Vec<TypeErr>> {
    match &_type.node {
        ASTNode::Type { id, generics } => {
            let (generics, generic_errs): (Vec<_>, Vec<_>) = generics
                .iter()
                .map(|generic| TypeName::try_from_node_pos(generic))
                .partition(Result::is_ok);
            if !generic_errs.is_empty() {
                return Err(generic_errs.into_iter().map(Result::unwrap_err).collect());
            }

            Ok((
                try_from_id(id.deref()).map_err(|err| vec![err])?,
                generics.into_iter().map(Result::unwrap).collect::<Vec<TypeName>>()
            ))
        }
        _ => Err(vec![TypeErr::new(Position::from(_type), "Expected class name")])
    }
}

fn get_fields_and_functions(
    statements: &[ASTNodePos]
) -> Result<(Vec<Field>, Vec<Function>), Vec<TypeErr>> {
    let (mut field_res, mut fun_res, mut errs) = (vec![], vec![], vec![]);
    statements.iter().for_each(|statement| match &statement.node {
        ASTNode::FunDef { .. } => fun_res.push(Function::try_from_node_pos(statement)),
        ASTNode::VariableDef { .. } => field_res.push(Field::try_from_node_pos(statement)),
        _ => errs.push(TypeErr::new(
            Position::from(statement),
            "Expected function or variable definition"
        ))
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
