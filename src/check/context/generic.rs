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
            Node::Block { statements } => {
                for module in statements {
                    match &module.node {
                        Node::Class { .. } | Node::TypeDef { .. } | Node::TypeAlias { .. } => {
                            types.insert(GenericClass::try_from(module)?);
                        }
                        Node::FunDef { .. } => {
                            functions.insert(GenericFunction::try_from(module)?);
                        }
                        Node::VariableDef { .. } => {
                            GenericFields::try_from(module)?.fields.iter().for_each(|ty| {
                                fields.insert(ty.clone());
                            });
                        }
                        _ => {}
                    }
                }
            }
            _ => return Err(vec![TypeErr::new(file.pos, "Expected file")]),
        }
    }

    Ok((types, fields, functions))
}
