use std::convert::TryFrom;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::collection::{constr_col, gen_collection_lookup};
use crate::check::constrain::generate::env::Environment;
use crate::check::context::{clss, Context};
use crate::check::context::clss::{FLOAT, INT, STRING};
use crate::check::context::function::{
    ADD, DIV, EQ, FDIV, GE, GEQ, LE, LEQ, MOD, MUL, POW, SQRT, SUB,
};
use crate::check::name::{Any, Name};
use crate::check::name::string_name::StringName;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub fn gen_op(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::In { left, right } => {
            let mut constr = constr_col(right, env, constr, None)?;
            let (mut constr, env) = gen_collection_lookup(left, right, env, &mut constr)?;
            let (mut constr, env) = generate(right, &env, ctx, &mut constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::Range { .. } => {
            constr.add(
                "range",
                &Expected::new(ast.pos, &Type { name: Name::from(clss::RANGE) }),
                &Expected::try_from((ast, &env.var_mappings))?,
            );
            constr_range(ast, env, ctx, constr, "range", true)
        }
        Node::Slice { .. } => {
            constr.add(
                "slice",
                &Expected::new(ast.pos, &Type { name: Name::from(clss::SLICE) }),
                &Expected::try_from((ast, &env.var_mappings))?,
            );
            constr_range(ast, env, ctx, constr, "slice", false)
        }

        Node::Real { .. } => primitive(ast, FLOAT, env, constr),
        Node::Int { .. } => primitive(ast, INT, env, constr),
        Node::ENum { .. } => primitive(ast, INT, env, constr),
        Node::Str { expressions, .. } => {
            let (mut constr, env) = gen_vec(expressions, env, ctx, constr)?;
            for expr in expressions {
                let c =
                    Constraint::stringy("string", &Expected::try_from((expr, &env.var_mappings))?);
                constr.add_constr(&c);
            }

            let name = Name::from(STRING);
            let left = Expected::try_from((ast, &env.var_mappings))?;
            constr.add("string", &left, &Expected::new(ast.pos, &Type { name }));
            Ok((constr, env))
        }
        Node::Bool { .. } => {
            constr.add_constr(&Constraint::truthy(
                "if else",
                &Expected::try_from((ast, &env.var_mappings))?,
            ));
            Ok((constr.clone(), env.clone()))
        }
        Node::Undefined => {
            constr.add_constr(&Constraint::undefined(
                "undefined",
                &Expected::try_from((ast, &env.var_mappings))?,
            ));
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
        Node::Neq { left, right } | Node::Eq { left, right } => {
            impl_bool_op(EQ, ast, left, right, env, ctx, constr)
        }

        Node::AddU { expr } | Node::SubU { expr } => generate(expr, env, ctx, constr),
        Node::Sqrt { expr } => {
            let ty = Type { name: Name::from(FLOAT) };
            constr.add(
                "square root",
                &Expected::try_from((ast, &env.var_mappings))?,
                &Expected::new(ast.pos, &ty),
            );

            let access = Expected::new(
                expr.pos,
                &Access {
                    entity: Box::new(Expected::try_from((expr, &env.var_mappings))?),
                    name: Box::from(Expected::new(
                        expr.pos,
                        &Function {
                            name: StringName::from(SQRT),
                            args: vec![Expected::try_from((expr, &env.var_mappings))?],
                        },
                    )),
                },
            );
            constr.add("square root", &Expected::try_from((ast, &env.var_mappings))?, &access);

            generate(expr, env, ctx, constr)
        }

        Node::BOneCmpl { expr } => {
            let left = Expected::try_from((expr, &env.var_mappings))?;
            constr.add("binary compliment", &left, &Expected::new(expr.pos, &Type { name: Name::any() }));
            generate(expr, env, ctx, constr)
        }
        Node::BAnd { left, right } | Node::BOr { left, right } | Node::BXOr { left, right } => {
            let l_exp = Expected::try_from((left, &env.var_mappings))?;
            constr.add("binary logical op", &l_exp, &Expected::new(left.pos, &Type { name: Name::any() }));

            let l_exp = Expected::try_from((right, &env.var_mappings))?;
            constr.add("binary logical op", &l_exp, &Expected::new(right.pos, &Type { name: Name::any() }));

            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::BLShift { left, right } | Node::BRShift { left, right } => {
            let l_exp = Expected::try_from((left, &env.var_mappings))?;
            constr.add("binary shift", &l_exp, &Expected::new(right.pos, &Type { name: Name::any() }));

            let name = Name::from(INT);
            let l_exp = Expected::try_from((right, &env.var_mappings))?;
            constr.add("binary shift", &l_exp, &Expected::new(right.pos, &Type { name }));

            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }

        Node::Is { left, right } | Node::IsN { left, right } => {
            let bool = Expected::new(ast.pos, &Type { name: Name::from(clss::BOOL) });
            constr.add("and", &Expected::try_from((ast, &env.var_mappings))?, &bool);
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::IsA { left, right } | Node::IsNA { left, right } => {
            let bool = Expected::new(ast.pos, &Type { name: Name::from(clss::BOOL) });
            constr.add("and", &Expected::try_from((ast, &env.var_mappings))?, &bool);
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }

        Node::Not { expr } => {
            let bool = Expected::new(ast.pos, &Type { name: Name::from(clss::BOOL) });
            constr.add("and", &Expected::try_from((ast, &env.var_mappings))?, &bool);
            constr.add_constr(&Constraint::truthy(
                "not",
                &Expected::try_from((expr, &env.var_mappings))?,
            ));
            generate(expr, env, ctx, constr)
        }
        Node::And { left, right } | Node::Or { left, right } => {
            let bool = Expected::new(ast.pos, &Type { name: Name::from(clss::BOOL) });
            constr.add("and", &Expected::try_from((ast, &env.var_mappings))?, &bool);

            constr.add_constr(&Constraint::truthy(
                "and",
                &Expected::try_from((left, &env.var_mappings))?,
            ));
            constr.add_constr(&Constraint::truthy(
                "and",
                &Expected::try_from((right, &env.var_mappings))?,
            ));
            let (mut constr, env) = generate(left, env, ctx, constr)?;
            generate(right, &env, ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Was expecting operation or primitive")]),
    }
}

pub fn constr_range(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
    range_slice: &str,
    contr_coll: bool,
) -> Constrained {
    let (from, to, step) = match &ast.node {
        Node::Range { from, to, step, .. } if range_slice == "range" => (from, to, step),
        Node::Slice { from, to, step, .. } if range_slice == "slice" => (from, to, step),
        _ => {
            let msg = format!("Expected {}, was {}", range_slice, ast.node);
            return Err(vec![TypeErr::new(ast.pos, &msg)]);
        }
    };

    let name = Name::from(INT);
    let int_exp = &Expected::new(from.pos, &Type { name });

    constr.add(
        &format!("{} from", range_slice),
        &Expected::try_from((from, &env.var_mappings))?,
        int_exp,
    );
    constr.add(
        &format!("{} to", range_slice),
        &Expected::try_from((to, &env.var_mappings))?,
        int_exp,
    );
    if let Some(step) = step {
        constr.add(
            &format!("{} step", range_slice),
            &Expected::try_from((step, &env.var_mappings))?,
            int_exp,
        );
    }

    if contr_coll {
        let col = Expected::new(ast.pos, &Collection { ty: Box::from(int_exp.clone()) });
        constr.add("range collection", &col, &Expected::try_from((ast, &env.var_mappings))?);
    }

    let (mut constr, env) = generate(from, env, ctx, constr)?;
    let (mut constr, env) = generate(to, &env, ctx, &mut constr)?;
    if let Some(step) = step {
        generate(step, &env, ctx, &mut constr)
    } else {
        Ok((constr, env))
    }
}

fn primitive(ast: &AST, ty: &str, env: &Environment, constr: &mut ConstrBuilder) -> Constrained {
    let name = Name::from(ty);
    constr.add(
        format!("{} primitive", ty).as_str(),
        &Expected::try_from((ast, &env.var_mappings))?,
        &Expected::new(ast.pos, &Type { name }),
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
    constr: &mut ConstrBuilder,
) -> Constrained {
    constr.add(
        format!("{} operation", fun).as_str(),
        &Expected::try_from((ast, &env.var_mappings))?,
        &Expected::new(
            left.pos,
            &Access {
                entity: Box::new(Expected::try_from((left, &env.var_mappings))?),
                name: Box::new(Expected::new(
                    left.pos,
                    &Function {
                        name: StringName::from(fun),
                        args: vec![
                            Expected::try_from((left, &env.var_mappings))?,
                            Expected::try_from((right, &env.var_mappings))?,
                        ],
                    },
                )),
            },
        ),
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
    constr: &mut ConstrBuilder,
) -> Constrained {
    constr.add(
        "bool operation",
        &Expected::try_from((ast, &env.var_mappings))?,
        &Expected::new(
            left.pos,
            &Access {
                entity: Box::new(Expected::try_from((left, &env.var_mappings))?),
                name: Box::new(Expected::new(
                    left.pos,
                    &Function {
                        name: StringName::from(fun),
                        args: vec![
                            Expected::try_from((left, &env.var_mappings))?,
                            Expected::try_from((right, &env.var_mappings))?,
                        ],
                    },
                )),
            },
        ),
    );
    let ty = Type { name: Name::from(clss::BOOL) };
    constr.add(
        "bool operation",
        &Expected::try_from((ast, &env.var_mappings))?,
        &Expected::new(ast.pos, &ty),
    );

    let (mut constr, env) = generate(left, env, ctx, constr)?;
    generate(right, &env, ctx, &mut constr)
}
