use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::definition::constrain_args;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_result::TypeErr;

pub fn gen_expr(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::AnonFun { args, body } => {
            // TODO generate constraint for anonymous function itself
            let (mut constr, env) = constrain_args(args, env, ctx, constr)?;
            generate(body, &env, ctx, &mut constr)
        }
        Node::Id { lit } if env.get_var_new(lit).is_some() => Ok((constr.clone(), env.clone())),
        Node::Id { lit } =>
            Err(vec![TypeErr::new(&ast.pos, &format!("Undefined variable: {}", lit))]),
        Node::Question { left, right } => {
            let nullable = Nullable { expect: Box::from(Expression { ast: *left.clone() }) };
            let l_exp = Expected::new(&left.pos, &nullable);
            let r_exp = Expected::new(&right.pos, &Expression { ast: *right.clone() });
            constr.add(&l_exp, &r_exp);

            let (mut constr, env) = generate(left, env, ctx, constr)?;
            generate(right, &env, ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Expected expression")])
    }
}
