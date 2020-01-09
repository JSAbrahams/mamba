use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::cons::Constraints;
use crate::type_checker::constraints::cons::Expect;
use crate::type_checker::constraints::Constrained;
use crate::type_checker::context::function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::ty;
use crate::type_checker::context::Context;
use crate::type_checker::environment::Environment;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::type_result::TypeErr;

pub fn generate(ast: &AST, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    match &ast.node {
        Node::File { modules, .. } => {
            let mut constr_env = (constr.clone(), env.clone());
            for module in modules {
                constr_env = generate(module, &env, &ctx, &constr)?;
            }
            Ok(constr_env)
        }

        Node::Block { statements } | Node::Script { statements } => {
            let mut constr_env = (constr.clone(), env.clone());
            for statement in statements {
                constr_env = generate(statement, &env, &ctx, &constr)?;
            }
            Ok(constr_env)
        }

        Node::Class { body, .. } =>
            if let Some(body) = body {
                match &body.node {
                    Node::Block { statements } => {
                        let mut constr_env = (constr.clone(), env.clone());
                        for statement in statements {
                            constr_env = generate(statement, &env, &ctx, &constr)?;
                        }
                        Ok(constr_env)
                    }
                    _ => Err(vec![TypeErr::new(&body.pos, "Expected code block")])
                }
            } else {
                Ok((constr.clone(), env.clone()))
            },

        Node::TypeDef { body, .. } =>
            if let Some(body) = body {
                generate(body, env, ctx, constr)
            } else {
                Ok((constr.clone(), env.clone()))
            },
        Node::TypeAlias { conditions, .. } => {
            let mut constr_env = (constr.clone(), env.clone());
            for cond in conditions {
                constr_env = generate(cond, &env, &ctx, &constr)?;
            }
            Ok(constr_env)
        }
        Node::Condition { cond, _else } => {
            let (constr, env) = generate(cond, env, ctx, constr)?;
            if let Some(el) = _else {
                generate(el, &env, ctx, &constr)
            } else {
                Ok((constr, env))
            }
        }

        Node::VariableDef { .. } => unimplemented!(),
        Node::FunDef { fun_args, ret_ty, body, .. } => {
            for fun_arg in fun_args {
                match &fun_arg.node {
                    Node::FunArg { .. } => unimplemented!(),
                    _ => return Err(vec![TypeErr::new(&fun_arg.pos, "Expected function argument")])
                }
            }

            match (ret_ty, body) {
                (Some(ret_ty), Some(body)) => {
                    let type_name = TypeName::try_from(ret_ty.deref())?;
                    let constr = constr
                        .add(&Expect::Expression { ast: body.deref().clone() }, &Expect::Type {
                            type_name
                        });
                    generate(body, &env, ctx, &constr)
                }
                _ => Ok((constr.clone(), env.clone()))
            }
        }
        Node::Reassign { left, right } => {
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Expression {
                    ast: right.deref().clone()
                });
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Mutable {
                    expect: Box::from(Expect::AnyExpression)
                });
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }

        Node::ConstructorCall { name, args } => {
            let type_name = TypeName::try_from(name.deref())?;
            // TODO lookup in environment if not in context
            let possible_constructor_args: HashSet<Vec<FunctionArg>> =
                ctx.lookup(&type_name, &ast.pos)?.constructor_args(&ast.pos)?;

            let constr = fun_args(ast, &possible_constructor_args, args, constr)?;
            let (constr, env) = gen_vec(args, env, ctx, &constr)?;
            generate(name, &env, ctx, &constr)
        }
        Node::FunctionCall { name, args } => {
            let type_name = TypeName::try_from(name.deref())?;
            // TODO lookup in environment if not in context
            let possible_fun_args: HashSet<Vec<FunctionArg>> =
                ctx.lookup_fun_args(&type_name, &ast.pos)?;

            let constr = fun_args(ast, &possible_fun_args, args, constr)?;
            let (constr, env) = gen_vec(args, env, ctx, &constr)?;
            generate(name, &env, ctx, &constr)
        }
        Node::PropertyCall { .. } => unimplemented!(),

        Node::TypeTup { .. } => unimplemented!(),
        Node::TypeUnion { .. } => unimplemented!(),
        Node::Type { .. } => unimplemented!(),
        Node::TypeFun { .. } => unimplemented!(),

        Node::AnonFun { .. } => unimplemented!(),
        Node::Raises { .. } => unimplemented!(),
        Node::Raise { .. } => unimplemented!(),
        Node::Handle { .. } => unimplemented!(),
        Node::With { .. } => unimplemented!(),
        Node::Id { lit } =>
            if env.vars.contains(lit) {
                Ok((constr.clone(), env.clone()))
            } else {
                Err(vec![TypeErr::new(&ast.pos, &format!("Unknown variable: {}", lit))])
            },

        Node::SetBuilder { .. } => unimplemented!(),
        Node::ListBuilder { .. } => unimplemented!(),
        Node::Set { elements } | Node::List { elements } =>
            if let Some(first) = elements.first() {
                let mut constr_env = (constr.clone(), env.clone());
                for element in elements {
                    constr_env.0 = constr_env
                        .0
                        .add(&Expect::Expression { ast: element.clone() }, &Expect::Expression {
                            ast: first.clone()
                        });
                    constr_env = generate(element, &env, &ctx, &constr)?;
                }
                Ok(constr_env)
            } else {
                Ok((constr.clone(), env.clone()))
            },
        Node::Tuple { elements } => {
            let mut constr_env = (constr.clone(), env.clone());
            for element in elements {
                constr_env = generate(element, &env, &ctx, &constr)?;
            }
            Ok(constr_env)
        }
        Node::Range { .. } => unimplemented!(),

        Node::Real { .. } => primitive(ast, ty::concrete::FLOAT_PRIMITIVE, env, constr),
        Node::Int { .. } => primitive(ast, ty::concrete::INT_PRIMITIVE, env, constr),
        Node::ENum { .. } => primitive(ast, ty::concrete::INT_PRIMITIVE, env, constr),
        Node::Str { .. } => primitive(ast, ty::concrete::STRING_PRIMITIVE, env, constr),
        Node::Bool { .. } => {
            let constr =
                constr.add(&Expect::Expression { ast: ast.deref().clone() }, &Expect::Truthy);
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
            let constr =
                constr.add(&Expect::Expression { ast: ast.deref().clone() }, &Expect::Truthy);
            let (constr, env) = generate(expr, env, ctx, &constr)?;
            Ok((constr, env))
        }
        Node::SubU { .. } => unimplemented!(),
        Node::Sqrt { expr } => {
            let constr = constr.add(
                &Expect::Expression { ast: expr.deref().clone() },
                &Expect::Implements {
                    name: String::from(function::concrete::SQRT),
                    args: vec![Expect::Expression { ast: expr.deref().clone() }]
                }
            );
            generate(expr, &env, ctx, &constr)
        }

        Node::BOneCmpl { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            generate(expr, &env, ctx, &constr)
        }
        Node::BAnd { left, right } | Node::BOr { left, right } | Node::BXOr { left, right } => {
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::AnyExpression);
            let constr = constr
                .add(&Expect::Expression { ast: right.deref().clone() }, &Expect::AnyExpression);
            let (constr, env) = generate(right, env, ctx, &constr)?;
            generate(left, &env, ctx, &constr)
        }

        Node::BLShift { left, right } | Node::BRShift { left, right } => {
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::AnyExpression);
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let constr = constr
                .add(&Expect::Expression { ast: right.deref().clone() }, &Expect::Type {
                    type_name
                });
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
            let constr =
                constr.add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::Truthy);
            generate(expr, env, ctx, &constr)
        }
        Node::And { left, right } | Node::Or { left, right } => {
            let constr =
                constr.add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Truthy);
            let constr =
                constr.add(&Expect::Expression { ast: right.deref().clone() }, &Expect::Truthy);
            let (constr, env) = generate(left, env, ctx, &constr)?;
            generate(right, &env, &ctx, &constr)
        }
        Node::IfElse { cond, then, _else } => {
            let constr =
                constr.add(&Expect::Expression { ast: cond.deref().clone() }, &Expect::Truthy);
            if let Some(_else) = _else {
                // TODO change constraint depending on whether we expect an expression or not
                let constr = constr
                    .add(&Expect::Expression { ast: then.deref().clone() }, &Expect::Expression {
                        ast: _else.deref().clone()
                    });
                let (constr, env) = generate(cond, env, ctx, &constr)?;
                let (constr, env) = generate(then, &env, ctx, &constr)?;
                generate(_else, &env, ctx, &constr)
            } else {
                let constr = constr
                    .add(&Expect::Expression { ast: then.deref().clone() }, &Expect::AnyExpression);
                let (constr, env) = generate(cond, env, ctx, &constr)?;
                generate(then, &env, ctx, &constr)
            }
        }

        Node::Match { .. } => unimplemented!(),
        Node::Case { .. } => unimplemented!(),

        Node::For { expr, col, body } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            let constr =
                constr.add(&Expect::Expression { ast: col.deref().clone() }, &Expect::Collection {
                    ty: Some(Box::from(Expect::Expression { ast: expr.deref().clone() }))
                });
            let (constr, env) = generate(expr, env, ctx, &constr)?;
            let (constr, env) = generate(col, &env, ctx, &constr)?;
            generate(body, &env, ctx, &constr)
        }
        Node::In { .. } => unimplemented!(),
        Node::Step { amount } => {
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            let constr = constr
                .add(&Expect::Expression { ast: amount.deref().clone() }, &Expect::Type {
                    type_name
                });
            Ok((constr, env.clone()))
        }
        Node::While { cond, body } => {
            let constr =
                constr.add(&Expect::Expression { ast: cond.deref().clone() }, &Expect::Truthy);
            generate(body, env, ctx, &constr)
        }

        Node::Return { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            let constr = constr
                .add(&Expect::Expression { ast: ast.deref().clone() }, &Expect::Expression {
                    ast: expr.deref().clone()
                });
            generate(expr, env, ctx, &constr)
        }

        Node::Question { left, right } => {
            let constr = constr
                .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Nullable {
                    expect: Box::from(Expect::AnyExpression)
                });
            let constr = constr
                .add(&Expect::Expression { ast: right.deref().clone() }, &Expect::AnyExpression);
            let (constr, env) = generate(left, env, ctx, &constr)?;
            generate(right, &env, ctx, &constr)
        }
        Node::QuestionOp { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            generate(expr, env, ctx, &constr)
        }

        Node::Print { expr } => {
            let constr = constr
                .add(&Expect::Expression { ast: expr.deref().clone() }, &Expect::AnyExpression);
            generate(expr, env, ctx, &constr)
        }

        _ => Ok((constr.clone(), env.clone()))
    }
}

fn gen_vec(asts: &Vec<AST>, env: &Environment, ctx: &Context, constr: &Constraints) -> Constrained {
    let mut constr_env = (constr.clone(), env.clone());
    for ast in asts {
        constr_env = generate(ast, &constr_env.1, ctx, &constr_env.0)?;
    }
    Ok(constr_env)
}

fn primitive(ast: &AST, ty: &str, env: &Environment, constr: &Constraints) -> Constrained {
    let type_name = TypeName::from(ty);
    let constr =
        constr.add(&Expect::Expression { ast: ast.deref().clone() }, &Expect::Type { type_name });
    Ok((constr, env.clone()))
}

fn fun_args(
    ast: &AST,
    possible: &HashSet<Vec<FunctionArg>>,
    args: &Vec<AST>,
    constr: &Constraints
) -> Result<Constraints, Vec<TypeErr>> {
    let mut constr = constr.clone();
    for fun_args in possible {
        for pair in fun_args.iter().zip_longest(args.iter()) {
            match pair {
                EitherOrBoth::Both(fun_arg, arg) =>
                    constr = constr.add(
                        &Expect::Expression { ast: arg.deref().clone() },
                        &Expect::Type {
                            type_name: fun_arg
                                .ty
                                .as_ref()
                                .ok_or_else(|| {
                                    TypeErr::new(&arg.pos, "Functions mut have type parameters")
                                })?
                                .clone()
                        }
                    ),
                EitherOrBoth::Left(fun_arg) if !fun_arg.has_default =>
                    return Err(vec![TypeErr::new(
                        &Position::new(&ast.pos.end, &ast.pos.end),
                        "Expected argument"
                    )]),
                EitherOrBoth::Right(arg) =>
                    return Err(vec![TypeErr::new(&arg.pos, "Unexpected argument")]),
                _ => {}
            }
        }
    }
    Ok(constr)
}

fn implements(
    fun: &str,
    left: &AST,
    right: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &Constraints
) -> Constrained {
    let constr = constr
        .add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Expression {
            ast: right.deref().clone()
        });
    let constr =
        constr.add(&Expect::Expression { ast: left.deref().clone() }, &Expect::Implements {
            name: String::from(fun),
            args: vec![Expect::Expression { ast: left.deref().clone() }, Expect::Expression {
                ast: right.deref().clone()
            }]
        });
    let (constr, env) = generate(left, env, ctx, &constr)?;
    generate(right, &env, ctx, &constr)
}
