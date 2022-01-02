use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::generate::Constrained;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub fn gen_ty(ast: &AST, _: &Environment, _: &Context, _: &ConstrBuilder) -> Constrained {
    match &ast.node {
        Node::QuestionOp { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Nullable type annotation cannot be top level")]),
        Node::TypeTup { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Type tuple annotation cannot be top level")]),
        Node::TypeUnion { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Type union annotation cannot be top level")]),
        Node::Type { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Type annotation cannot be top level")]),
        Node::TypeFun { .. } =>
            Err(vec![TypeErr::new(&ast.pos, "Type annotation function cannot be top level")]),
        _ => Err(vec![TypeErr::new(&ast.pos, "Expected type annotation")])
    }
}
