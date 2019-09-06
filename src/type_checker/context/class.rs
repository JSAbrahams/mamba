use crate::common::position::Position;
use crate::parser::ast::{ASTNode, ASTNodePos};
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
        ASTNode::Class { .. } => unimplemented!(),
        _ => panic!()
    }
}

fn get_type_def(type_def: &ASTNodePos) -> TypeResult {
    match &type_def.node {
        ASTNode::TypeDef { .. } => unimplemented!(),
        _ => panic!()
    }
}
