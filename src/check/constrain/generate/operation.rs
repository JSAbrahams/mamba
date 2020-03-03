use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::generate::collection::{constr_col, gen_collection_lookup};
use crate::check::constrain::generate::{gen_vec, generate};
use crate::check::constrain::Constrained;
use crate::check::context::clss::{FLOAT_PRIMITIVE, INT_PRIMITIVE, STRING_PRIMITIVE};
use crate::check::context::function::{ADD, DIV, EQ, FDIV, GE, GEQ, LE, LEQ, MOD, MUL, POW, SQRT,
                                      SUB};
use crate::check::context::name::{DirectName, NameUnion};
use crate::check::context::{clss, Context};
use crate::check::env::Environment;
use crate::check::result::TypeErr;
use crate::parse::ast::{Node, AST};

pub fn gen_op(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::In { left, right } => {
            let mut constr = constr_col(right, constr)?;
            let (mut constr, env) = gen_collection_lookup(left, &right, env, &mut constr)?;
            let (mut constr, env) = generate(right, &env, ctx, &mut constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::Range { from, to, step, .. } => {
            let name = NameUnion::from(clss::INT_PRIMITIVE);
            let int_exp = &Expected::new(&from.pos, &Type { name });

            constr.add("range from", &Expected::try_from(from)?, &int_exp);
            constr.add("range to", &Expected::try_from(to)?, &int_exp);
            let col = Expected::new(&ast.pos, &Collection { ty: Box::from(int_exp.clone()) });
            constr.add("range collection", &col, &Expected::try_from(ast)?);

            if let Some(step) = step {
                constr.add("range step", &Expected::try_from(step)?, &int_exp);
            }

            let (mut constr, env) = generate(from, env, ctx, constr)?;
            let (mut constr, env) = generate(to, &env, ctx, &mut constr)?;
            if let Some(step) = step {
                generate(step, &env, ctx, &mut constr)
            } else {
                Ok((constr, env))
            }
        }

        Node::Real { .. } => primitive(ast, FLOAT_PRIMITIVE, env, constr),
        Node::Int { .. } => primitive(ast, INT_PRIMITIVE, env, constr),
        Node::ENum { .. } => primitive(ast, INT_PRIMITIVE, env, constr),
        Node::Str { expressions, .. } => {
            let (mut constr, env) = gen_vec(expressions, env, ctx, constr)?;
            for expr in expressions {
                let c = Constraint::stringy("string", &Expected::try_from(expr)?);
                constr.add_constr(&c);
            }

            let name = NameUnion::from(STRING_PRIMITIVE);
            let left = Expected::try_from(ast)?;
            constr.add("string", &left, &Expected::new(&ast.pos, &Type { name }));
            Ok((constr, env))
        }
        Node::Bool { .. } => {
            constr.add_constr(&Constraint::truthy("if else", &Expected::try_from(ast)?));
            Ok((constr.clone(), env.clone()))
        }

        Node::Add { left, right } => impl_magic(ADD, ast, left, right, env, ctx, constr),
        Node::Sub { left, right } => impl_magic(SUB, ast, left, right, env, ctx, constr),
        Node::Mul { left, right } => impl_magic(MUL, ast, left, right, env, ctx, constr),
        Node::Div { left, right } => impl_magic(DIV, ast, left, right, env, ctx, constr),
        Node::FDiv { left, right } => impl_magic(FDIV, ast, left, right, env, ctx, constr),
        Node::Pow { left, right } => impl_magic(POW, ast, left, right, env, ctx, constr),
        Node::Mod { left, right } => impl_magic(MOD, ast, left, right, env, ctx, constr),

        Node::Le { left, right } => impl_bool_op(LE, ast, left, right, env, ctx, constr),
        Node::Ge { left, right } => impl_bool_op(GE, ast, left, right, env, ctx, constr),
        Node::Leq { left, right } => impl_bool_op(LEQ, ast, left, right, env, ctx, constr),
        Node::Geq { left, right } => impl_bool_op(GEQ, ast, left, right, env, ctx, constr),
        Node::Neq { left, right } | Node::Eq { left, right } =>
            impl_bool_op(EQ, ast, left, right, env, ctx, constr),

        Node::AddU { expr } | Node::SubU { expr } => generate(expr, env, ctx, constr),
        Node::Sqrt { expr } => {
            let ty = Type { name: NameUnion::from(clss::FLOAT_PRIMITIVE) };
            constr.add("square root", &Expected::try_from(ast)?, &Expected::new(&ast.pos, &ty));

            let access = Expected::new(&expr.pos, &Access {
                entity: Box::new(Expected::try_from(expr)?),
                name:   Box::from(Expected::new(&expr.pos, &Function {
                    name: DirectName::from(SQRT),
                    args: vec![Expected::try_from(expr)?]
                }))
            });
            constr.add("square root", &Expected::try_from(ast)?, &access);

            generate(expr, &env, ctx, constr)
        }

        Node::BOneCmpl { expr } => {
            let left = Expected::try_from(expr)?;
            constr.add("binary compliment", &left, &Expected::new(&expr.pos, &ExpressionAny));
            generate(expr, &env, ctx, constr)
        }
        Node::BAnd { left, right } | Node::BOr { left, right } | Node::BXOr { left, right } => {
            let l_exp = Expected::try_from(left)?;
            constr.add("binary logical op", &l_exp, &Expected::new(&left.pos, &ExpressionAny));

            let l_exp = Expected::try_from(right)?;
            constr.add("binary logical op", &l_exp, &Expected::new(&right.pos, &ExpressionAny));

            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::BLShift { left, right } | Node::BRShift { left, right } => {
            let l_exp = Expected::try_from(left)?;
            constr.add("binary shift", &l_exp, &Expected::new(&right.pos, &ExpressionAny));

            let name = NameUnion::from(clss::INT_PRIMITIVE);
            let l_exp = Expected::try_from(right)?;
            constr.add("binary shift", &l_exp, &Expected::new(&right.pos, &Type { name }));

            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }

        Node::Is { left, right } | Node::IsN { left, right } => {
            let bool =
                Expected::new(&ast.pos, &Type { name: NameUnion::from(clss::BOOL_PRIMITIVE) });
            constr.add("and", &Expected::try_from(ast)?, &bool);
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::IsA { left, right } | Node::IsNA { left, right } => {
            let bool =
                Expected::new(&ast.pos, &Type { name: NameUnion::from(clss::BOOL_PRIMITIVE) });
            constr.add("and", &Expected::try_from(ast)?, &bool);
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }

        Node::Not { expr } => {
            let bool =
                Expected::new(&ast.pos, &Type { name: NameUnion::from(clss::BOOL_PRIMITIVE) });
            constr.add("and", &Expected::try_from(ast)?, &bool);
            constr.add_constr(&Constraint::truthy("not", &Expected::try_from(expr)?));
            generate(expr, env, ctx, constr)
        }
        Node::And { left, right } | Node::Or { left, right } => {
            let bool =
                Expected::new(&ast.pos, &Type { name: NameUnion::from(clss::BOOL_PRIMITIVE) });
            constr.add("and", &Expected::try_from(ast)?, &bool);

            constr.add_constr(&Constraint::truthy("and", &Expected::try_from(left)?));
            constr.add_constr(&Constraint::truthy("and", &Expected::try_from(right)?));
            let (mut constr, env) = generate(left, env, ctx, constr)?;
            generate(right, &env, &ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Was expecting operation or primitive")])
    }
}

fn primitive(ast: &AST, ty: &str, env: &Environment, constr: &mut ConstrBuilder) -> Constrained {
    let name = NameUnion::from(ty);
    constr.add(
        format!("{} primitive", ty).as_str(),
        &Expected::try_from(ast)?,
        &Expected::new(&ast.pos, &Type { name })
    );
    Ok((constr.clone(), env.clone()))
}

fn impl_magic(
    fun: &str,
    ast: &AST,
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    constr.add(
        format!("{} operation", fun).as_str(),
        &Expected::try_from(ast)?,
        &Expected::new(&left.pos, &Access {
            entity: Box::new(Expected::try_from(left)?),
            name:   Box::new(Expected::new(&left.pos, &Function {
                name: DirectName::from(fun),
                args: vec![Expected::try_from(left)?, Expected::try_from(right)?]
            }))
        })
    );

    let (mut constr, env) = generate(left, env, ctx, constr)?;
    generate(right, &env, ctx, &mut constr)
}

fn impl_bool_op(
    fun: &str,
    ast: &AST,
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    constr.add(
        "bool operation",
        &Expected::try_from(ast)?,
        &Expected::new(&left.pos, &Access {
            entity: Box::new(Expected::try_from(left)?),
            name:   Box::new(Expected::new(&left.pos, &Function {
                name: DirectName::from(fun),
                args: vec![Expected::try_from(left)?, Expected::try_from(right)?]
            }))
        })
    );
    let ty = Type { name: NameUnion::from(clss::BOOL_PRIMITIVE) };
    constr.add("bool operation", &Expected::try_from(ast)?, &Expected::new(&ast.pos, &ty));

    let (mut constr, env) = generate(left, env, ctx, constr)?;
    generate(right, &env, ctx, &mut constr)
}
