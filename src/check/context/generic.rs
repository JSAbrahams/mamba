use std::collections::HashSet;
use std::convert::TryFrom;

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::context::function::generic::GenericFunction;
use crate::check::result::TypeResult;
use crate::parse::ast::{Node, OptAST, AST};

pub fn generics(
    asts: &[AST],
) -> TypeResult<(
    HashSet<GenericClass>,
    HashSet<GenericField>,
    HashSet<GenericFunction>,
)> {
    let mut types = HashSet::new();
    let mut fields = HashSet::new();
    let mut functions = HashSet::new();

    for ast in asts {
        match &ast.node {
            Node::Block { statements } => {
                for ast in statements {
                    single_ast(ast, &mut types, &mut fields, &mut functions)?
                }
            }
            _ => single_ast(ast, &mut types, &mut fields, &mut functions)?,
        };
    }

    Ok((types, fields, functions))
}

fn single_ast(
    ast: &AST,
    types: &mut HashSet<GenericClass>,
    fields: &mut HashSet<GenericField>,
    functions: &mut HashSet<GenericFunction>,
) -> TypeResult<()> {
    match &ast.node {
        Node::Class { .. } | Node::TypeDef { .. } | Node::TypeAlias { .. } => {
            types.insert(GenericClass::try_from(ast)?);
        }
        Node::FunDef { .. } => {
            functions.insert(GenericFunction::try_from(ast)?);
        }
        Node::VariableDef { .. } => {
            GenericFields::try_from(ast)?.fields.iter().for_each(|ty| {
                fields.insert(ty.clone());
            });
        }
        Node::Import {
            from,
            import,
            alias,
        } => from_import(from, import, alias)?.into_iter().for_each(|t| {
            types.insert(t);
        }),
        _ => {}
    };

    Ok(())
}

/// From import.
///
/// A more elaborate import system will extract the signature of the class.
fn from_import(
    _from: &OptAST,
    import: &[AST],
    alias: &[Option<AST>],
) -> TypeResult<Vec<GenericClass>> {
    let mut classes = vec![];
    for pair in import.iter().zip(alias) {
        match pair {
            (import, None) => classes.push(GenericClass::try_from_id(import)?),
            (_, Some(alias)) => classes.push(GenericClass::try_from_id(alias)?),
        }
    }

    Ok(classes)
}
