use crate::parser::ast::AST;
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;

pub fn gen_vec(
    asts: &Vec<AST>,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    let mut constr_env = (constr.clone(), env.clone());
    for ast in asts {
        constr_env = generate(ast, &constr_env.1, ctx, &constr_env.0)?;
    }
    Ok(constr_env)
}
