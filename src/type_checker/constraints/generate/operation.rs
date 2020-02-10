use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::collection::{constr_col, gen_collection_lookup};
use crate::type_checker::constraints::generate::{gen_vec, generate};
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function::concrete::*;
use crate::type_checker::context::ty::concrete::{FLOAT_PRIMITIVE, INT_PRIMITIVE, STRING_PRIMITIVE};
use crate::type_checker::context::{ty, Context};
use crate::type_checker::environment::Environment;
use crate::type_checker::ty_name::TypeName;

pub fn gen_op(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    match &ast.node {
        Node::In { left, right } => {
            let (mut constr, _) = constr_col(right, constr);
            let (mut constr, env) = gen_collection_lookup(left, &right, env, &mut constr)?;
            let (mut constr, env) = generate(right, &env, ctx, &mut constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::Range { from, to, step: Some(step), .. } => {
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let l_exp = Expected::from(from);
            constr.add(&l_exp, &Expected::new(&from.pos, &Type { type_name: type_name.clone() }));

            let l_exp = Expected::from(to);
            constr.add(&l_exp, &Expected::new(&to.pos, &Type { type_name: type_name.clone() }));

            let l_exp = Expected::from(step);
            constr.add(&l_exp, &Expected::new(&step.pos, &Type { type_name }));

            let (mut constr, env) = generate(from, env, ctx, constr)?;
            let (mut constr, env) = generate(to, &env, ctx, &mut constr)?;
            generate(step, &env, ctx, &mut constr)
        }
        Node::Range { from, to, .. } => {
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let l_exp = Expected::from(from);
            constr.add(&l_exp, &Expected::new(&from.pos, &Type { type_name: type_name.clone() }));

            let l_exp = Expected::from(to);
            constr.add(&l_exp, &Expected::new(&to.pos, &Type { type_name }));

            let (mut constr, env) = generate(from, env, ctx, constr)?;
            generate(to, &env, ctx, &mut constr)
        }

        Node::Real { .. } => primitive(ast, FLOAT_PRIMITIVE, env, constr),
        Node::Int { .. } => primitive(ast, INT_PRIMITIVE, env, constr),
        Node::ENum { .. } => primitive(ast, INT_PRIMITIVE, env, constr),
        Node::Str { expressions, .. } => {
            let (mut constr, env) = gen_vec(expressions, env, ctx, constr)?;
            for expr in expressions {
                constr.add(&Expected::from(expr), &Expected::new(&expr.pos, &Stringy))
            }
            let type_name = TypeName::from(STRING_PRIMITIVE);
            let left = Expected::from(ast);
            constr.add(&left, &Expected::new(&ast.pos, &Type { type_name }));
            Ok((constr, env))
        }
        Node::Bool { .. } => {
            let left = Expected::from(ast);
            constr.add(&left, &Expected::new(&ast.pos, &Truthy));
            Ok((constr.clone(), env.clone()))
        }

        Node::Add { left, right } => impl_magic(ADD, ast, left, right, env, ctx, constr),
        Node::Sub { left, right } => impl_magic(SUB, ast, left, right, env, ctx, constr),
        Node::Mul { left, right } => impl_magic(MUL, ast, left, right, env, ctx, constr),
        Node::Div { left, right } => impl_magic(DIV, ast, left, right, env, ctx, constr),
        Node::FDiv { left, right } => impl_magic(FDIV, ast, left, right, env, ctx, constr),
        Node::Pow { left, right } => impl_magic(POW, ast, left, right, env, ctx, constr),
        Node::Mod { left, right } => impl_magic(MOD, ast, left, right, env, ctx, constr),
        Node::Neq { left, right } => impl_magic(NEQ, ast, left, right, env, ctx, constr),

        Node::Le { left, right } => impl_bool_op(LE, ast, left, right, env, ctx, constr),
        Node::Ge { left, right } => impl_bool_op(GE, ast, left, right, env, ctx, constr),
        Node::Leq { left, right } => impl_bool_op(LEQ, ast, left, right, env, ctx, constr),
        Node::Geq { left, right } => impl_bool_op(GEQ, ast, left, right, env, ctx, constr),
        Node::Eq { left, right } => impl_bool_op(EQ, ast, left, right, env, ctx, constr),

        Node::AddU { expr } | Node::SubU { expr } => generate(expr, env, ctx, constr),
        Node::Sqrt { expr } => {
            let expr_expt = Expected::from(expr);
            let implements = Expected::new(&expr.pos, &Implements {
                type_name: TypeName::from(SQRT),
                args:      vec![Expected::from(expr)]
            });

            let ty = Type { type_name: TypeName::from(ty::concrete::FLOAT_PRIMITIVE) };
            constr.add(&Expected::from(ast), &Expected::new(&ast.pos, &ty));
            constr.add(&expr_expt, &implements);

            generate(expr, &env, ctx, constr)
        }

        Node::BOneCmpl { expr } => {
            let left = Expected::from(expr);
            constr.add(&left, &Expected::new(&expr.pos, &ExpressionAny));
            generate(expr, &env, ctx, constr)
        }
        Node::BAnd { left, right } | Node::BOr { left, right } | Node::BXOr { left, right } => {
            let l_exp = Expected::from(left);
            constr.add(&l_exp, &Expected::new(&left.pos, &ExpressionAny));

            let l_exp = Expected::from(right);
            constr.add(&l_exp, &Expected::new(&right.pos, &ExpressionAny));

            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::BLShift { left, right } | Node::BRShift { left, right } => {
            let l_exp = Expected::from(left);
            constr.add(&l_exp, &Expected::new(&right.pos, &ExpressionAny));

            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let l_exp = Expected::from(right);
            constr.add(&l_exp, &Expected::new(&right.pos, &Type { type_name }));

            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }

        Node::Is { left, right } | Node::IsN { left, right } => {
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }
        Node::IsA { left, right } | Node::IsNA { left, right } => {
            let (mut constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &mut constr)
        }

        Node::Not { expr } => {
            let left = Expected::from(expr);
            constr.add(&left, &Expected::new(&expr.pos, &Truthy));
            generate(expr, env, ctx, constr)
        }
        Node::And { left, right } | Node::Or { left, right } => {
            let l_exp = Expected::from(left);
            constr.add(&l_exp, &Expected::new(&left.pos, &Truthy));

            let l_exp = Expected::from(right);
            constr.add(&l_exp, &Expected::new(&right.pos, &Truthy));

            let (mut constr, env) = generate(left, env, ctx, constr)?;
            generate(right, &env, &ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Was expecting operation or primitive")])
    }
}

fn primitive(ast: &AST, ty: &str, env: &Environment, constr: &mut ConstrBuilder) -> Constrained {
    let type_name = TypeName::from(ty);
    constr.add(&Expected::from(ast), &Expected::new(&ast.pos, &Type { type_name }));
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
    let exp_implements = Expected::new(&left.pos, &Implements {
        type_name: TypeName::from(fun),
        args:      vec![Expected::from(left), Expected::from(right)]
    });
    // type of first arg and return of operator
    constr.add(&Expected::from(left), &exp_implements);
    // type of whole expression and first arg the same
    constr.add(&Expected::from(ast), &Expected::from(left));

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
    let exp_implements = Expected::new(&left.pos, &Implements {
        type_name: TypeName::from(fun),
        args:      vec![Expected::from(left), Expected::from(right)]
    });
    // type of first arg and return of operator
    constr.add(&Expected::from(left), &exp_implements);
    let ty = Type { type_name: TypeName::from(ty::concrete::BOOL_PRIMITIVE) };
    constr.add(&Expected::from(ast), &Expected::new(&ast.pos, &ty));

    let (mut constr, env) = generate(left, env, ctx, constr)?;
    generate(right, &env, ctx, &mut constr)
}
