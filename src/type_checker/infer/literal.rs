use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic::type_name::GenericType;
use crate::type_checker::context::{concrete, Context};
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_literal(ast: &AST, env: &Environment, ctx: &Context, _: &State) -> InferResult {
    Ok((
        match &ast.node {
            Node::Real { .. } => ctx.lookup(&GenericType::new(concrete::FLOAT_PRIMITIVE), &ast.pos),
            Node::Int { .. } => ctx.lookup(&GenericType::new(concrete::INT_PRIMITIVE), &ast.pos),
            Node::ENum { .. } => ctx.lookup(&GenericType::new(concrete::ENUM_PRIMITIVE), &ast.pos),
            Node::Str { .. } => ctx.lookup(&GenericType::new(concrete::STRING_PRIMITIVE), &ast.pos),
            Node::Bool { .. } => ctx.lookup(&GenericType::new(concrete::BOOL_PRIMITIVE), &ast.pos),
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected control flow")])
        }?,
        env.clone()
    ))
}
