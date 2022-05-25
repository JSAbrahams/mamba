use std::collections::HashSet;
use std::convert::TryFrom;

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::context::function::generic::GenericFunction;
use crate::check::result::{TypeErr, TypeResult};
use crate::parse::ast::{AST, Node};

pub fn generics(
    files: &[AST],
) -> TypeResult<(HashSet<GenericClass>, HashSet<GenericField>, HashSet<GenericFunction>)> {
    let mut types = HashSet::new();
    let mut fields = HashSet::new();
    let mut functions = HashSet::new();

    for file in files {
        match &file.node {
            Node::File { statements: modules, .. } => {
                for module in modules {
                    match &module.node {
                        Node::Class { .. } | Node::TypeDef { .. } | Node::TypeAlias { .. } => {
                            let generic_type = GenericClass::try_from(module)?;
                            types.insert(generic_type);
                        }
                        Node::FunDef { .. } => {
                            let generic_type = GenericFunction::try_from(module)?;
                            functions.insert(generic_type);
                        }
                        Node::VariableDef { .. } => {
                            let generic_type = GenericFields::try_from(module)?;
                            generic_type.fields.iter().for_each(|ty| {
                                fields.insert(ty.clone());
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => return Err(vec![TypeErr::new(&file.pos, "Expected file")]),
        }
    }

    Ok((types, fields, functions))
}
