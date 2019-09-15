use std::collections::HashSet;

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::infer_type::InferType;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;

pub fn infer_block(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Script { statements } => {
            let ast = Box::from(AST {
                pos:  ast.pos.clone(),
                node: Node::Block { statements: statements.clone() }
            });
            infer(&ast, env, ctx, state)
        }
        Node::Block { statements } => {
            let mut last_stmt_type = None;
            let mut raises = HashSet::new();
            let mut block_env = env.clone();

            for statement in statements {
                let (mut statement_type, new_env) =
                    infer(&Box::from(statement.clone()), &block_env, ctx, state)?;
                statement_type.raises.iter().for_each(|err| {
                    raises.insert(err.clone());
                });
                last_stmt_type = Some(statement_type);
                block_env = new_env.clone();
            }

            let infer_type = last_stmt_type.unwrap_or_else(|| InferType::new(None));
            Ok((infer_type.raises(raises), env.clone()))
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected code block")])
    }
}