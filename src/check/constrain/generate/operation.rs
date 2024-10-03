use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::{gen_vec, generate, Constrained};
use crate::check::context::clss::{BOOL, FLOAT, INT, RANGE, SLICE, STRING};
use crate::check::context::function::python::CONTAINS;
use crate::check::context::function::python::{
    ADD, DIV, EQ, FDIV, GE, GEQ, LE, LEQ, MOD, MUL, NEQ, POW, SUB,
};
use crate::check::context::function::SQRT;
use crate::check::context::{Context, LookupClass};
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::check::name::Name;
use crate::check::result::TypeErr;
use crate::parse::ast::{Node, AST};

pub fn gen_op(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::In { left, right } => gen_magic(CONTAINS, ast, right, left, env, ctx, constr),
        Node::Range { .. } => {
            gen_primitive(ast, RANGE, env, constr)?;
            gen_range(ast, env, ctx, constr, "range")
        }
        Node::Slice { .. } => {
            gen_primitive(ast, SLICE, env, constr)?;
            gen_range(ast, env, ctx, constr, "slice")
        }

        Node::Real { .. } => gen_primitive(ast, FLOAT, env, constr),
        Node::Int { .. } => gen_primitive(ast, INT, env, constr),
        Node::ENum { .. } => gen_primitive(ast, INT, env, constr),
        Node::Str { expressions, .. } => {
            gen_vec(expressions, env, false, ctx, constr)?;
            for expr in expressions {
                constr.add_constr(&Constraint::stringy("string", &Expected::from(expr)), env);
            }
            gen_primitive(ast, STRING, env, constr)
        }
        Node::Bool { .. } => {
            constr.add_constr(&Constraint::truthy("bool", &Expected::from(ast)), env);
            gen_primitive(ast, BOOL, env, constr)
        }
        Node::Undefined => {
            constr.add_constr(
                &Constraint::undefined("undefined", &Expected::from(ast)),
                env,
            );
            Ok(env.clone())
        }

        Node::Add { left, right } => gen_magic(ADD, ast, left, right, env, ctx, constr),
        Node::Sub { left, right } => gen_magic(SUB, ast, left, right, env, ctx, constr),
        Node::Mul { left, right } => gen_magic(MUL, ast, left, right, env, ctx, constr),
        Node::Div { left, right } => gen_magic(DIV, ast, left, right, env, ctx, constr),
        Node::FDiv { left, right } => gen_magic(FDIV, ast, left, right, env, ctx, constr),
        Node::Pow { left, right } => gen_magic(POW, ast, left, right, env, ctx, constr),
        Node::Mod { left, right } => gen_magic(MOD, ast, left, right, env, ctx, constr),

        Node::Le { left, right } => gen_magic(LE, ast, left, right, env, ctx, constr),
        Node::Ge { left, right } => gen_magic(GE, ast, left, right, env, ctx, constr),
        Node::Leq { left, right } => gen_magic(LEQ, ast, left, right, env, ctx, constr),
        Node::Geq { left, right } => gen_magic(GEQ, ast, left, right, env, ctx, constr),
        Node::Neq { left, right } => gen_magic(NEQ, ast, left, right, env, ctx, constr),
        Node::Eq { left, right } => gen_magic(EQ, ast, left, right, env, ctx, constr),

        Node::AddU { expr } | Node::SubU { expr } => generate(expr, env, ctx, constr),
        Node::Sqrt { expr } => {
            let ty = Type {
                name: Name::from(FLOAT),
            };
            constr.add(
                "square root",
                &Expected::from(ast),
                &Expected::new(ast.pos, &ty),
                env,
            );

            let access = Expected::new(
                expr.pos,
                &Access {
                    entity: Box::new(Expected::from(expr)),
                    name: Box::from(Expected::new(
                        expr.pos,
                        &Function {
                            name: StringName::from(SQRT),
                            args: vec![Expected::from(expr)],
                        },
                    )),
                },
            );

            constr.add("square root", &Expected::from(ast), &access, env);
            generate(expr, env, ctx, constr)
        }

        Node::BOneCmpl { expr } => {
            constr.add(
                "binary compliment",
                &Expected::from(expr),
                &Expected::any(expr.pos),
                env,
            );
            generate(expr, env, ctx, constr)?;
            Ok(env.clone())
        }
        Node::BAnd { left, right } | Node::BOr { left, right } | Node::BXOr { left, right } => {
            constr.add(
                "binary logical op",
                &Expected::from(left),
                &Expected::any(left.pos),
                env,
            );
            constr.add(
                "binary logical op",
                &Expected::from(right),
                &Expected::any(right.pos),
                env,
            );

            bin_op(left, right, env, ctx, constr)
        }
        Node::BLShift { left, right } | Node::BRShift { left, right } => {
            constr.add(
                "binary shift",
                &Expected::from(left),
                &Expected::any(right.pos),
                env,
            );

            let name = Name::from(INT);
            let l_exp = Expected::from(right);
            constr.add(
                "binary shift",
                &l_exp,
                &Expected::new(right.pos, &Type { name }),
                env,
            );

            bin_op(left, right, env, ctx, constr)
        }

        Node::Is { left, right } | Node::IsN { left, right } => {
            let bool = Expected::new(
                ast.pos,
                &Type {
                    name: Name::from(BOOL),
                },
            );
            constr.add("and", &Expected::from(ast), &bool, env);
            bin_op(left, right, env, ctx, constr)
        }
        Node::IsA { left, right } | Node::IsNA { left, right } => {
            if let Node::Id { .. } = right.node {
                let class_name = TrueName::try_from(right)?;
                ctx.class(&class_name, right.pos)?;

                generate(left, env, ctx, constr)?;
                generate(right, &env.is_def_mode(true), ctx, constr)?;
                Ok(env.clone())
            } else {
                let msg = format!("Expected identifier: '{}'", right.node);
                Err(vec![TypeErr::new(ast.pos, &msg)])
            }
        }

        Node::Not { expr } => {
            let bool = Expected::new(
                ast.pos,
                &Type {
                    name: Name::from(BOOL),
                },
            );
            constr.add("not", &Expected::from(ast), &bool, env);
            constr.add_constr(&Constraint::truthy("not", &Expected::from(expr)), env);

            generate(expr, env, ctx, constr)?;
            Ok(env.clone())
        }
        Node::And { left, right } | Node::Or { left, right } => {
            let bool = Expected::new(
                ast.pos,
                &Type {
                    name: Name::from(BOOL),
                },
            );
            constr.add("and", &Expected::from(ast), &bool, env);

            constr.add_constr(&Constraint::truthy("and", &Expected::from(left)), env);
            constr.add_constr(&Constraint::truthy("and", &Expected::from(right)), env);
            bin_op(left, right, env, ctx, constr)
        }

        _ => Err(vec![TypeErr::new(
            ast.pos,
            "Was expecting operation or primitive",
        )]),
    }
}

pub fn gen_range(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
    range_slice: &str,
) -> Constrained {
    let (from, to, step) = match &ast.node {
        Node::Range { from, to, step, .. } if range_slice == "range" => (from, to, step),
        Node::Slice { from, to, step, .. } if range_slice == "slice" => (from, to, step),
        _ => {
            let msg = format!("Expected {range_slice}, was {}", ast.node);
            return Err(vec![TypeErr::new(ast.pos, &msg)]);
        }
    };

    let int_exp = &Expected::new(
        from.pos,
        &Type {
            name: Name::from(INT),
        },
    );
    constr.add(
        &format!("{range_slice} from"),
        &Expected::from(from),
        int_exp,
        env,
    );
    constr.add(
        &format!("{range_slice} to"),
        &Expected::from(to),
        int_exp,
        env,
    );
    if let Some(step) = step {
        constr.add(
            &format!("{range_slice} step"),
            &Expected::from(step),
            int_exp,
            env,
        );
    }

    generate(from, env, ctx, constr)?;
    generate(to, env, ctx, constr)?;
    if let Some(step) = step {
        generate(step, env, ctx, constr)?;
    }
    Ok(env.clone())
}

fn gen_primitive(
    ast: &AST,
    ty: &str,
    env: &Environment,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let msg = format!("{ty} primitive");
    constr.add(
        &msg,
        &Expected::from(ast),
        &Expected::new(
            ast.pos,
            &Type {
                name: Name::from(ty),
            },
        ),
        env,
    );
    Ok(env.clone())
}

pub fn gen_magic(
    fun: &str,
    ast: &AST,
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let res = gen_vec(
        &[right.clone(), left.clone()],
        env,
        env.is_def_mode,
        ctx,
        constr,
    )?;
    constr.add(
        format!("{fun} operation").as_str(),
        &Expected::from(ast),
        &access(fun, left, right),
        env,
    );
    Ok(res)
}

fn access(fun: &str, left: &AST, right: &AST) -> Expected {
    let name = StringName::from(fun);
    Expected::new(
        left.pos,
        &Access {
            entity: Box::new(Expected::from(left)),
            name: Box::new(Expected::new(
                left.pos,
                &Function {
                    name,
                    args: vec![Expected::from(left), Expected::from(right)],
                },
            )),
        },
    )
}

fn bin_op(
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    gen_vec(&[right.clone(), left.clone()], env, false, ctx, constr)
}
