use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, generate};
use crate::check::constrain::generate::collection::gen_collection_lookup;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::Context;
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub fn gen_flow(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Handle { expr_or_stmt, cases } => {
            let (raises, errs): (Vec<Result<_, _>>, Vec<Result<_, _>>) = cases.iter().map(|c| match &c.node {
                Node::Case { cond, .. } => {
                    match &cond.node {
                        Node::ExpressionType { ty: Some(ty), .. } => TrueName::try_from(ty)
                            .map_err(|errs| errs.first().expect("At least one").clone()),
                        other => {
                            let msg = format!("Expected type identifier, was {other}");
                            Err(TypeErr::new(cond.pos, &msg))
                        }
                    }
                }
                other => Err(TypeErr::new(c.pos, &format!("Expected case, was {other}")))
            }).partition(Result::is_ok);

            if !errs.is_empty() { return Err(errs.into_iter().map(Result::unwrap_err).collect()); }
            let raises = raises.into_iter().map(Result::unwrap).collect();

            let raises_before = env.raises_caught.clone();
            let outer_env = generate(expr_or_stmt, &env.raises_caught(&raises), ctx, constr)?
                .raises_caught(&raises_before);

            constrain_cases(ast, &None, cases, &outer_env, ctx, constr)?;
            Ok(outer_env.clone())
        }

        Node::IfElse { cond, then, el: Some(el) } => {
            let left = Expected::try_from((cond, &env.var_mappings))?;
            constr.add_constr(&Constraint::truthy("if else", &left));
            generate(cond, env, ctx, constr)?;

            constr.new_set(true);
            let then_env = generate(then, env, ctx, constr)?;
            constr.exit_set(then.pos)?;
            constr.new_set(true);
            let else_env = generate(el, env, ctx, constr)?;
            constr.exit_set(el.pos)?;

            if env.is_expr {
                let if_expr = Expected::try_from((ast, &env.var_mappings))?;
                let then = Expected::try_from((then, &then_env.var_mappings))?;
                let el = Expected::try_from((el, &else_env.var_mappings))?;

                constr.add("if then branch", &if_expr, &then);
                constr.add("if else branch", &if_expr, &el);
            }

            Ok(env.union(&then_env.intersect(&else_env)))
        }
        Node::IfElse { cond, then, .. } => {
            let left = Expected::try_from((cond, &env.var_mappings))?;
            constr.add_constr(&Constraint::truthy("if else", &left));

            generate(cond, env, ctx, constr)?;
            constr.new_set(true);
            generate(then, env, ctx, constr)?;
            constr.exit_set(then.pos)?;
            Ok(env.clone())
        }

        Node::Case { .. } => Err(vec![TypeErr::new(ast.pos, "Case cannot be top level")]),
        Node::Match { cond, cases } => {
            let outer_env = generate(cond, env, ctx, constr)?;
            constrain_cases(ast, &Some(*cond.clone()), cases, &outer_env, ctx, constr)?;
            Ok(env.clone())
        }

        Node::For { expr, col, body } => {
            constr.new_set(true);
            let col_env = generate(col, env, ctx, constr)?;

            let is_define_mode = col_env.is_def_mode;
            let lookup_env = gen_collection_lookup(expr, col, &col_env.is_def_mode(true), constr)?;

            generate(body, &lookup_env.in_loop().is_def_mode(is_define_mode), ctx, constr)?;
            constr.exit_set(ast.pos)?;
            Ok(env.clone())
        }
        Node::While { cond, body } => {
            constr.new_set(true);
            let cond_exp = Expected::try_from((cond, &env.var_mappings))?;
            constr.add_constr(&Constraint::truthy("while condition", &cond_exp));

            generate(cond, env, ctx, constr)?;
            generate(body, &env.in_loop(), ctx, constr)?;
            constr.exit_set(ast.pos)?;
            Ok(env.clone())
        }

        Node::Break | Node::Continue if env.in_loop => Ok(env.clone()),
        Node::Break | Node::Continue => Err(vec![TypeErr::new(ast.pos, "Cannot be outside loop")]),

        _ => Err(vec![TypeErr::new(ast.pos, "Expected control flow")])
    }
}

fn constrain_cases(ast: &AST, expr: &Option<AST>, cases: &Vec<AST>, env: &Environment, ctx: &Context, constr: &mut ConstrBuilder) -> Constrained<()> {
    let is_define_mode = env.is_def_mode;
    let exp_ast = Expected::try_from((ast, &env.var_mappings))?;

    for case in cases {
        match &case.node {
            Node::Case { cond, body } => {
                constr.new_set(true);

                let cond_env = generate(cond, &env.is_def_mode(true), ctx, constr)?;
                generate(body, &cond_env.is_def_mode(is_define_mode), ctx, constr)?;

                if let Node::ExpressionType { expr: ref cond, .. } = cond.node {
                    if let Some(expr) = expr {
                        constr.add("match expression and arm condition",
                                   &Expected::try_from((expr, &env.var_mappings))?,
                                   &Expected::try_from((cond, &env.var_mappings))?,
                        );
                    }
                }

                let exp_body = Expected::try_from((body, &cond_env.var_mappings))?;
                constr.add("match arm body", &exp_body, &exp_ast);
                constr.exit_set(case.pos)?;
            }
            _ => return Err(vec![TypeErr::new(case.pos, "Expected case")])
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::check::constrain::constraint::builder::ConstrBuilder;
    use crate::check::constrain::generate::env::Environment;
    use crate::check::constrain::generate::generate;
    use crate::check::context::Context;
    use crate::parse::parse;

    #[test]
    fn if_else_env_empty() {
        let src = "if True then 10 else 20";
        let ast = parse(src).unwrap();
        let env = generate(&ast, &Environment::default(), &Context::default(), &mut ConstrBuilder::new()).unwrap();

        assert!(env.var_mappings.is_empty());
    }
}
