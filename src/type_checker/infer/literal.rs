use crate::parser::ast::{Node, AST};
use crate::type_checker::context::ty::concrete;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::infer_type::InferType;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn infer_literal(ast: &AST, env: &Environment, ctx: &Context) -> InferResult {
    Ok((
        InferType::from(&match &ast.node {
            Node::Real { .. } => ctx.lookup(&TypeName::from(concrete::FLOAT_PRIMITIVE), &ast.pos),
            Node::Int { .. } => ctx.lookup(&TypeName::from(concrete::INT_PRIMITIVE), &ast.pos),
            Node::ENum { .. } => ctx.lookup(&TypeName::from(concrete::ENUM_PRIMITIVE), &ast.pos),
            Node::Str { expressions, .. } => {
                for expression in expressions {
                    let (expr_ty, _) = infer(expression, env, ctx)?;
                    let expr_ty = expr_ty.expr_ty(&expression.pos)?;
                    expr_ty.fun("__str__", &[], &expression.pos)?;
                }
                ctx.lookup(&TypeName::from(concrete::STRING_PRIMITIVE), &ast.pos)
            }
            Node::Bool { .. } => ctx.lookup(&TypeName::from(concrete::BOOL_PRIMITIVE), &ast.pos),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
        }?),
        env.clone()
    ))
}
