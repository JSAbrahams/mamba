use crate::parser::ast::{Node, AST};
use crate::type_checker::context::{concrete, Context};
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::infer_type::InferType;
use crate::type_checker::infer::InferResult;
use crate::type_checker::type_result::TypeErr;

pub fn infer_literal(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    let infer_type = match &ast.node {
        Node::Real { .. } => ctx.lookup_primitive(concrete::FLOAT_PRIMITIVE, &ast.pos),
        Node::Int { .. } => ctx.lookup_primitive(concrete::INT_PRIMITIVE, &ast.pos),
        Node::ENum { .. } => ctx.lookup_primitive(concrete::ENUM_PRIMITIVE, &ast.pos),
        Node::Str { .. } => ctx.lookup_primitive(concrete::STRING_PRIMITIVE, &ast.pos),
        Node::Bool { .. } => ctx.lookup_primitive(concrete::BOOL_PRIMITIVE, &ast.pos),
        _ => Err(TypeErr::new(&ast.pos, "Expected control flow"))
    }
    .map_err(|e| vec![e])?;

    Ok((InferType::from(infer_type), env.clone()))
}
