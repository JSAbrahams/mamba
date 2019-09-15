use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_assign(
    ast: &Box<AST>,
    env: &Environment,
    ctx: &Context,
    state: &State
) -> InferResult {
    match &ast.node {
        Node::Reassign { left, right } =>
            if let (Some(left_ty), left_env) = infer(left, env, ctx, state)? {
                if !left_ty.mutable {
                    return Err(vec![TypeErr::new(&left.pos, "Cannot be assigned to")]);
                }

                if let (Some(right_ty), right_env) = infer(right, &left_env, ctx, state)? {
                    // TODO override type of identifier in environment if some
                    unimplemented!()
                } else {
                    Err(vec![TypeErr::new(&right.pos, "Must be expression")])
                }
            } else {
                Err(vec![TypeErr::new(&left.pos, "Must be expression")])
            },
        // TODO use forward and private, and get rid of ofmut
        Node::VariableDef { id_maybe_type, expression, .. } => match id_maybe_type.node {
            Node::IdType { mutable, _type, .. } => match (_type, expression) {
                (Some(ty), Some(expr)) => unimplemented!(),
                (None, Some(expr)) => unimplemented!(),
                (Some(ty), None) => unimplemented!(),
                (None, None) => unimplemented!()
            },
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected identifier")])
        },
        Node::FunArg { .. } => unimplemented!(),
        Node::FunDef { fun_args, ret_ty, raises, body, .. } => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected variable manipulation")])
    }
}
