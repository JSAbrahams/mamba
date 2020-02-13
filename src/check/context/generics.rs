use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;

use crate::check::checker_result::{TypeErr, TypeResult};
use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::ty::generic::GenericType;
use crate::check::CheckInput;
use crate::parse::ast::{Node, AST};

pub fn generics(
    files: &[CheckInput]
) -> TypeResult<(HashSet<GenericType>, HashSet<GenericField>, HashSet<GenericFunction>)> {
    let mut types = HashSet::new();
    let mut fields = HashSet::new();
    let mut functions = HashSet::new();

    for (file, source, path) in files {
        match &file.node {
            Node::File { pure, modules, .. } =>
                for module in modules {
                    match &module.node {
                        Node::Class { .. } | Node::TypeDef { .. } | Node::TypeAlias { .. } => {
                            let generic_type: Result<_, Vec<TypeErr>> =
                                GenericType::try_from(module)
                                    .and_then(|ty| ty.all_pure(*pure))
                                    .map_err(|errs| {
                                        errs.into_iter()
                                            .map(|e| e.into_with_source(source, path))
                                            .collect()
                                    });
                            types.insert(generic_type?);
                        }
                        Node::Script { statements } => {
                            let (generic_fields, generic_functions) =
                                get_functions_and_fields(statements, source, path)?;
                            fields = fields.union(&generic_fields).cloned().collect();
                            functions = functions.union(&generic_functions).cloned().collect();
                        }
                        _ => {} // TODO process imports
                    }
                },
            _ => return Err(vec![TypeErr::new(&file.pos, "Expected file")])
        }
    }

    Ok((types, fields, functions))
}

fn get_functions_and_fields(
    statements: &[AST],
    source: &Option<String>,
    path: &Option<PathBuf>
) -> TypeResult<(HashSet<GenericField>, HashSet<GenericFunction>)> {
    let mut fields: HashSet<GenericField> = HashSet::new();
    let mut functions: HashSet<GenericFunction> = HashSet::new();

    for stmt in statements {
        match &stmt.node {
            Node::FunDef { .. } => {
                let generic_function: Result<_, Vec<TypeErr>> = GenericFunction::try_from(stmt)
                    .and_then(|f| f.in_class(None, false, &stmt.pos))
                    .map_err(|e| e.into_iter().map(|e| e.into_with_source(source, path)).collect());
                functions.insert(generic_function?);
            }
            Node::VariableDef { .. } => {
                let generic_fields = GenericFields::try_from(stmt)
                    .map_err(|e| {
                        e.into_iter()
                            .map(|e| e.into_with_source(source, path))
                            .collect::<Vec<TypeErr>>()
                    })?
                    .fields;
                let generic_fields: HashSet<_> = generic_fields
                    .into_iter()
                    .map(|f| f.in_class(None, false, &stmt.pos))
                    .collect::<Result<_, _>>()?;
                fields = fields.union(&generic_fields).cloned().collect();
            }
            _ => {}
        }
    }

    Ok((fields, functions))
}
