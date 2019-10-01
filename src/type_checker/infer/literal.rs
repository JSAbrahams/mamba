use crate::parser::ast::{Node, AST};
use crate::type_checker::context::concrete::type_name::TypeName;
use crate::type_checker::context::{concrete, Context};
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_literal(ast: &AST, env: &Environment, ctx: &Context, _: &State) -> InferResult {
    Ok((
        match &ast.node {
            Node::Real { .. } =>
                ctx.lookup(&TypeName::new(concrete::FLOAT_PRIMITIVE, &vec![]), &ast.pos),
            Node::Int { .. } =>
                ctx.lookup(&TypeName::new(concrete::INT_PRIMITIVE, &vec![]), &ast.pos),
            Node::ENum { .. } =>
                ctx.lookup(&TypeName::new(concrete::ENUM_PRIMITIVE, &vec![]), &ast.pos),
            Node::Str { .. } =>
                ctx.lookup(&TypeName::new(concrete::STRING_PRIMITIVE, &vec![]), &ast.pos),
            Node::Bool { .. } =>
                ctx.lookup(&TypeName::new(concrete::BOOL_PRIMITIVE, &vec![]), &ast.pos),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
        }?,
        env.clone()
    ))
}
