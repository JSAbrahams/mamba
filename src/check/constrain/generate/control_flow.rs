use std::collections::HashSet;
use std::convert::TryFrom;

use crate::check::constrain::constraint::{Constraint, ConstrVariant};
use crate::check::constrain::constraint::builder::ConstrBuilder;
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
            let raises: HashSet<TrueName> = cases.iter().flat_map(|c| match &c.node {
                Node::Case { cond, .. } => match &cond.node {
                    Node::ExpressionType { expr, .. } => if let Node::Id { lit } = &expr.node {
                        Some(TrueName::from(lit.as_str()))
                    } else { None }
                    _ => None
                },
                _ => None
            }).collect();

            let raises_before = env.raises_caught.clone();
            let (mut constr, outer_env) = generate(expr_or_stmt, &env.accounted_for_raises(&raises), ctx, constr)?;
            let outer_env = outer_env.accounted_for_raises(&raises_before);

            for case in cases {
                match &case.node {
                    Node::Case { cond, body } => {
                        let define_mode = outer_env.is_define_mode;
                        constr.new_set(true);

                        let (mut inner_constr, cond_env) = generate(cond, &outer_env.define_mode(true), ctx, &mut constr)?;
                        let cond_env = cond_env.define_mode(define_mode);

                        inner_constr.add(
                            "handle arm body",
                            &Expected::try_from((body, &cond_env.var_mappings))?,
                            &Expected::try_from((ast, &outer_env.var_mappings))?,
                        );

                        let (inner_constr, _) = generate(body, &cond_env, ctx, &mut inner_constr)?;
                        constr = inner_constr;

                        constr.exit_set(case.pos)?;
                    }
                    _ => return Err(vec![TypeErr::new(case.pos, "Expected case")])
                }
            }

            Ok((constr, outer_env))
        }

        Node::IfElse { cond, then, el: Some(el) } => {
            constr.new_set(true);
            let left = Expected::try_from((cond, &env.var_mappings))?;
            constr.add_constr(&Constraint::truthy("if else", &left));
            let (mut constr, _) = generate(cond, env, ctx, constr)?;

            constr.new_set(true);
            let (mut constr, then_env) = generate(then, env, ctx, &mut constr)?;
            constr.exit_set(then.pos)?;
            constr.new_set(true);
            let (mut constr, else_env) = generate(el, env, ctx, &mut constr)?;
            constr.exit_set(el.pos)?;

            if env.exp_expression {
                let if_expr = Expected::try_from((ast, &env.var_mappings))?;
                let then = Expected::try_from((then, &env.var_mappings))?;
                let el = Expected::try_from((el, &env.var_mappings))?;

                let then_constr = Constraint::new_variant("if then branch", &if_expr, &then, &ConstrVariant::Left);
                constr.add_constr(&then_constr);
                let else_constr = Constraint::new_variant("if else branch", &if_expr, &el, &ConstrVariant::Left);
                constr.add_constr(&else_constr);
            }

            constr.exit_set(ast.pos)?;
            Ok((constr, env.union(&then_env.intersect(&else_env))))
        }
        Node::IfElse { cond, then, .. } => {
            constr.new_set(true);
            let left = Expected::try_from((cond, &env.var_mappings))?;
            constr.add_constr(&Constraint::truthy("if else", &left));
            let (mut constr, env) = generate(cond, env, ctx, constr)?;

            let (mut constr, _) = generate(then, &env, ctx, &mut constr)?;
            constr.exit_set(then.pos)?;
            Ok((constr, env))
        }

        Node::Case { .. } => Err(vec![TypeErr::new(ast.pos, "Case cannot be top level")]),
        Node::Match { cond, cases } => {
            let (mut constr, outer_env) = generate(cond, &env, ctx, constr)?;

            for case in cases {
                match &case.node {
                    Node::Case { cond, body } => {
                        let define_mode = outer_env.is_define_mode;
                        constr.new_set(true);

                        let (mut inner_constr, cond_env) = generate(cond, &outer_env.define_mode(true), ctx, &mut constr)?;
                        let cond_env = cond_env.define_mode(define_mode);

                        inner_constr.add(
                            "match body",
                            &Expected::try_from((body, &cond_env.var_mappings))?,
                            &Expected::try_from((ast, &outer_env.var_mappings))?,
                        );

                        let (inner_constr, _) = generate(body, &cond_env, ctx, &mut inner_constr)?;
                        constr = inner_constr;

                        constr.exit_set(case.pos)?;
                    }
                    _ => return Err(vec![TypeErr::new(case.pos, "Expected case")])
                }
            }

            Ok((constr, outer_env))
        }

        Node::For { expr, col, body } => {
            constr.new_set(true);
            let (mut constr, for_env) = generate(col, env, ctx, constr)?;

            let is_define_mode = for_env.is_define_mode;
            let (mut constr, for_env) =
                gen_collection_lookup(expr, col, &for_env.define_mode(true), &mut constr)?;
            let (mut constr, _) =
                generate(body, &for_env.in_loop().define_mode(is_define_mode), ctx, &mut constr)?;

            constr.exit_set(ast.pos)?;
            Ok((constr, env.clone()))
        }
        Node::While { cond, body } => {
            constr.new_set(true);
            let left = Expected::try_from((cond, &env.var_mappings))?;
            constr.add_constr(&Constraint::truthy("if else", &left));
            let (mut constr, env) = generate(cond, env, ctx, constr)?;
            let (mut constr, _) = generate(body, &env.in_loop(), ctx, &mut constr)?;
            constr.exit_set(ast.pos)?;
            Ok((constr, env))
        }

        Node::Break | Node::Continue =>
            if env.in_loop {
                Ok((constr.clone(), env.clone()))
            } else {
                Err(vec![TypeErr::new(ast.pos, "Cannot be outside loop")])
            },

        _ => Err(vec![TypeErr::new(ast.pos, "Expected control flow")])
    }
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
        let (_, env) = generate(&ast, &Environment::default(), &Context::default(), &mut ConstrBuilder::new()).unwrap();

        assert!(env.var_mappings.is_empty());
    }
}
