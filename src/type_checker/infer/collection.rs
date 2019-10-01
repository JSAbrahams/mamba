use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::environment::expression_type::ExpressionType;
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
                env = new_env;
                errors.append(&mut el_ty.raises);
                types.push(ActualType::Single { expr_ty: el_ty.expr_ty(&el.pos)? });
            }

            let actual_ty = ActualType::Tuple { types };
            let expr_ty = ExpressionType::from(&actual_ty);
            Ok((InferType::from(&expr_ty), env))
        }
        Node::Set { .. } | Node::List { .. } => unimplemented!(),

        Node::ListBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),
        Node::SetBuilder { .. } => Err(vec![TypeErr::new(&ast.pos, "Not implemented")]),

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected collection")])
    }
}
