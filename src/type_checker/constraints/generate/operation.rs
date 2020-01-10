use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect::{AnyExpr, Collection, Expression, Implements,
                                                     Truthy, Type};
use crate::type_checker::constraints::generate::generate;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::{function, ty, Context};
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn gen_op(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::In { left, right } => {
            let constr = constr.add(&Expression { ast: *right.clone() }, &Collection {
                ty: Some(Box::from(Expression { ast: *left.clone() }))
            });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }
        Node::Range { from, to, inclusive, step: Some(step) } => {
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let constr = constr
                .add(&Expression { ast: *from.clone() }, &Type { type_name: type_name.clone() })
                .add(&Expression { ast: *to.clone() }, &Type { type_name: type_name.clone() })
                .add(&Expression { ast: *step.clone() }, &Type { type_name });

            let (constr, env) = generate(from, env, ctx, &constr)?;
            let (constr, env) = generate(to, &env, ctx, &constr)?;
            generate(step, &env, ctx, &constr)
        }
        Node::Range { from, to, inclusive, .. } => {
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let constr = constr
                .add(&Expression { ast: *from.clone() }, &Type { type_name: type_name.clone() })
                .add(&Expression { ast: *to.clone() }, &Type { type_name });
            let (constr, env) = generate(from, env, ctx, &constr)?;
            generate(to, &env, ctx, &constr)
        }

        Node::Real { .. } => primitive(ast, ty::concrete::FLOAT_PRIMITIVE, env, constr),
        Node::Int { .. } => primitive(ast, ty::concrete::INT_PRIMITIVE, env, constr),
        Node::ENum { .. } => primitive(ast, ty::concrete::INT_PRIMITIVE, env, constr),
        Node::Str { .. } => primitive(ast, ty::concrete::STRING_PRIMITIVE, env, constr),
        Node::Bool { .. } => {
            let constr = constr.add(&Expression { ast: ast.clone() }, &Truthy);
            Ok((constr, env.clone()))
        }

        Node::Add { left, right } =>
            implements(function::concrete::ADD, left, right, env, ctx, constr),
        Node::Sub { left, right } =>
            implements(function::concrete::SUB, left, right, env, ctx, constr),
        Node::Mul { left, right } =>
            implements(function::concrete::MUL, left, right, env, ctx, constr),
        Node::Div { left, right } =>
            implements(function::concrete::DIV, left, right, env, ctx, constr),
        Node::FDiv { left, right } =>
            implements(function::concrete::FDIV, left, right, env, ctx, constr),
        Node::Pow { left, right } =>
            implements(function::concrete::POW, left, right, env, ctx, constr),
        Node::Le { left, right } =>
            implements(function::concrete::LE, left, right, env, ctx, constr),
        Node::Ge { left, right } =>
            implements(function::concrete::GE, left, right, env, ctx, constr),
        Node::Leq { left, right } =>
            implements(function::concrete::LEQ, left, right, env, ctx, constr),
        Node::Geq { left, right } =>
            implements(function::concrete::GEQ, left, right, env, ctx, constr),
        Node::Eq { left, right } =>
            implements(function::concrete::EQ, left, right, env, ctx, constr),
        Node::Mod { left, right } =>
            implements(function::concrete::MOD, left, right, env, ctx, constr),
        Node::Neq { left, right } =>
            implements(function::concrete::NEQ, left, right, env, ctx, constr),
        Node::AddU { expr } => {
            let (constr, env) = generate(expr, env, ctx, &constr)?;
            Ok((constr, env))
        }
        Node::SubU { .. } => unimplemented!(),
        Node::Sqrt { expr } => {
            let constr = constr.add(&Expression { ast: *expr.clone() }, &Implements {
                name: String::from(function::concrete::SQRT),
                args: vec![Expression { ast: *expr.clone() }]
            });
            generate(expr, &env, ctx, &constr)
        }

        Node::BOneCmpl { expr } => {
            let constr = constr.add(&Expression { ast: *expr.clone() }, &AnyExpr);
            generate(expr, &env, ctx, &constr)
        }
        Node::BAnd { left, right } | Node::BOr { left, right } | Node::BXOr { left, right } => {
            let constr = constr.add(&Expression { ast: *left.clone() }, &AnyExpr);
            let constr = constr.add(&Expression { ast: *right.clone() }, &AnyExpr);
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }

        Node::BLShift { left, right } | Node::BRShift { left, right } => {
            let constr = constr.add(&Expression { ast: *left.clone() }, &AnyExpr);
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let constr = constr.add(&Expression { ast: *right.clone() }, &Type { type_name });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }

        Node::Is { left, right }
        | Node::IsN { left, right }
        | Node::IsA { left, right }
        | Node::IsNA { left, right } => {
            let (constr, env) = generate(right, env, ctx, constr)?;
            generate(left, &env, ctx, &constr)
        }

        Node::Not { expr } => {
            let constr = constr.add(&Expression { ast: *expr.clone() }, &Truthy);
            generate(expr, env, ctx, &constr)
        }
        Node::And { left, right } | Node::Or { left, right } => {
            let constr = constr.add(&Expression { ast: *left.clone() }, &Truthy);
            let constr = constr.add(&Expression { ast: *right.clone() }, &Truthy);
            let (constr, env) = generate(left, env, ctx, &constr)?;
            generate(right, &env, &ctx, &constr)
        }

        _ => Err(vec![TypeErr::new(&ast.pos, "Was expecting operation or primitive")])
    }
}

fn primitive(ast: &AST, ty: &str, env: &Environment, constr: &Constraints) -> Constrained {
    let type_name = TypeName::from(ty);
    let constr = constr.add(&Expression { ast: ast.clone() }, &Type { type_name });
    Ok((constr, env.clone()))
}

fn implements(
    fun: &str,
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    let constr = constr.add(&Expression { ast: left.clone() }, &Expression { ast: right.clone() });
    let constr = constr.add(&Expression { ast: left.clone() }, &Implements {
        name: String::from(fun),
        args: vec![Expression { ast: left.clone() }, Expression { ast: right.clone() }]
    });
    let (constr, env) = generate(left, env, ctx, &constr)?;
    generate(right, &env, ctx, &constr)
}
