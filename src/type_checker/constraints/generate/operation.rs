use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::builder::ConstrBuilder;
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::generate::collection::{gen_collection, gen_collection_lookup};
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
            let (mut constr, col) = gen_collection(right, constr);
            let (constr, env) = gen_collection_lookup(left, &col, env, &mut constr)?;
            Ok((constr, env))
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

        Node::Add { left, right } => implements(ADD, left, right, env, ctx, constr),
        Node::Sub { left, right } => implements(SUB, left, right, env, ctx, constr),
        Node::Mul { left, right } => implements(MUL, left, right, env, ctx, constr),
        Node::Div { left, right } => implements(DIV, left, right, env, ctx, constr),
        Node::FDiv { left, right } => implements(FDIV, left, right, env, ctx, constr),
        Node::Pow { left, right } => implements(POW, left, right, env, ctx, constr),
        Node::Le { left, right } => implements(LE, left, right, env, ctx, constr),
        Node::Ge { left, right } => implements(GE, left, right, env, ctx, constr),
        Node::Leq { left, right } => implements(LEQ, left, right, env, ctx, constr),
        Node::Geq { left, right } => implements(GEQ, left, right, env, ctx, constr),
        Node::Eq { left, right } => implements(EQ, left, right, env, ctx, constr),
        Node::Mod { left, right } => implements(MOD, left, right, env, ctx, constr),
        Node::Neq { left, right } => implements(NEQ, left, right, env, ctx, constr),
        Node::AddU { expr } | Node::SubU { expr } => generate(expr, env, ctx, constr),
        Node::Sqrt { expr } => {
            let left = Expected::from(expr);
            let right = Expected::new(&expr.pos, &Implements {
                type_name: TypeName::from(SQRT),
                args:      vec![Expected::from(expr)]
            });
            constr.add(&left, &right);
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

fn implements(
    fun: &str,
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder
) -> Constrained {
    let r_exp = Expected::new(&left.pos, &Implements {
        type_name: TypeName::from(fun),
        args:      vec![Expected::from(left), Expected::from(right)]
    });
    constr.add(&Expected::from(left), &r_exp);

    let (mut constr, env) = generate(left, env, ctx, constr)?;
    generate(right, &env, ctx, &mut constr)
}
