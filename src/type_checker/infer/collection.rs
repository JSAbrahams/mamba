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

pub fn infer_coll(ast: &AST, env: &Environment, ctx: &Context, state: &State) -> InferResult {
    match &ast.node {
        Node::Tuple { elements } => {
            let mut env = env.clone();
            let mut errors = vec![];
            let mut types = vec![];
            for el in elements {
                let (mut el_ty, new_env) = infer(el, &env, ctx, state)?;
                errors.append(&mut el_ty.raises);
                types.push(el_ty.expr_ty(&el.pos)?);
                env = new_env;
            }

            let actual_ty = ActualType::Tuple { types };
            let mut_ty = MutableType::from(&actual_ty);
            let expr_ty = ExpressionType::from(&mut_ty);
            Ok((InferType::from(&expr_ty), env))
        }
        Node::Set { .. } | Node::List { .. } => unimplemented!(),

        Node::ListBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),
        Node::SetBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}
