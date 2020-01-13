use std::convert::TryFrom;

use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{Expression, Type};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn gen_ty(ast: &AST, _: &Environment, _: &Context, _: &Constraints) -> Constrained {
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

pub fn constrain_ty(
    expr: &AST,
    ty: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    let type_name = TypeName::try_from(ty)?;
    let constr = constr.add(&Expression { ast: expr.clone() }, &Type { type_name });
    generate(expr, &env, ctx, &constr)
}
