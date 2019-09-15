use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_block(ast: &Box<AST>, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Block { statements } => {
            let mut types = vec![];
            let mut env = env;
            let mut state = state;
            for statement in statements {
                let (statement_type, new_env, new_state) =
                    infer(&Box::from(statement.clone()), env, ctx, state)?;
                types.push((statement_type, statement.pos));
                env = &new_env.clone();
                state = &new_state.clone();
            }

            // TODO check if all type inferred

            Ok((types.last().and_then(|(expr_ty, _)| expr_ty.clone()), env.clone(), state.clone()))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected code block")])
    }
}
