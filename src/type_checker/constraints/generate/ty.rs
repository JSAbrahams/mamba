use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{AnyExpr, Expression};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_ty(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::TypeTup { .. } => unimplemented!(),
        Node::TypeUnion { .. } => unimplemented!(),
        Node::Type { .. } => unimplemented!(),
        Node::TypeFun { .. } => unimplemented!(),

        Node::QuestionOp { expr } => {
            let constr = constr.add(&Expression { ast: *expr.clone() }, &AnyExpr);
            generate(expr, env, ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected type annotation")])
    }
}
