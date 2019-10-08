use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::mutable_type::MutableType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::environment::infer_type::InferType;
use crate::type_checker::environment::state::State;
use crate::type_checker::environment::Environment;
use crate::type_checker::infer::{infer, InferResult};
use crate::type_checker::type_result::TypeErr;
use std::collections::HashSet;

pub fn infer_coll(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Tuple { elements } => {
            let mut env = env.clone();
            let mut types = vec![];
            let mut raises = HashSet::new();
            for element in elements {
                let (mut ty, new_env) = infer(element, &env, ctx, state)?;
                types.push(ty.expr_ty(&element.pos)?);
                raises = raises.union(&ty.raises).cloned().collect();
                env = new_env;
            }

            let actual_ty = ActualType::Tuple { types };
            let mutable_ty = MutableType::from(&actual_ty);
            let expr_ty = ExpressionType::from(&mutable_ty);
            let ty = InferType::from(&expr_ty);
            Ok((ty.add_raises(&raises), env))
        }
        Node::Set { .. } | Node::List { .. } => unimplemented!(),

        Node::ListBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),
        Node::SetBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),

        Node::In { .. } => unimplemented!(),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}
