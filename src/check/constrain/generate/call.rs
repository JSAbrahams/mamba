use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::collection::constr_col;
use crate::check::constrain::generate::env::Environment;
use crate::check::context::{arg, clss, Context, function, LookupClass, LookupFunction};
use crate::check::context::arg::FunctionArg;
use crate::check::ident::{IdentiCall, Identifier};
use crate::check::name::{Empty, Name};
use crate::check::name::string_name::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};
use crate::parse::ast::node_op::NodeOp;

pub fn gen_call(
    ast: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    match &ast.node {
        Node::Reassign { left, right, op } => {
            let identifier = check_reassignable(left)?;
            check_iden_mut(&identifier, env, left.pos)?;

            if let NodeOp::Assign = op {
                let env: Environment = identifier
                    .all_calls()
                    .iter()
                    .flat_map(|call| call.without_obj(arg::SELF, left.pos))
                    .flat_map(|identi_call| match identi_call {
                        IdentiCall::Iden(var) => Some(var),
                        _ => None,
                    })
                    .fold(env.clone(), |env, self_var| env.assigned_to(&self_var));

                constr.add(
                    "reassign",
                    &Expected::try_from((left, &env.var_mappings))?,
                    &Expected::try_from((right, &env.var_mappings))?,
                );
                let (mut constr, _) = generate(right, &env, ctx, constr)?;
                generate(left, &env, ctx, &mut constr)
            } else {
                reassign_op(ast, left, right, op, env, ctx, constr)
            }
        }
        Node::FunctionCall { name, args } => {
            let f_name = StringName::try_from(name)?;
            let (mut constr, env) = gen_vec(args, env, ctx, constr)?;

            if f_name == StringName::from(function::PRINT) {
                let args = args
                    .iter()
                    .map(|arg| Expected::try_from((arg, &env.var_mappings)))
                    .collect::<TypeResult<Vec<Expected>>>()?;
                let args: Vec<Constraint> =
                    args.iter().map(|exp| Constraint::stringy("print", exp)).collect();
                let constr = args.iter().fold(constr.clone(), |mut acc, a| {
                    acc.add_constr(a);
                    acc
                });
                return Ok((constr, env));
            } else if let Some(functions) = env.get_var(&f_name.name) {
                if !f_name.generics.is_empty() {
                    let msg = "Anonymous function call cannot have generics";
                    return Err(vec![TypeErr::new(name.pos, msg)]);
                }

                for (_, fun_exp) in functions {
                    let last_pos = args.last().map_or_else(|| name.pos, |a| a.pos);
                    let args = args
                        .iter()
                        .map(|a| Expected::try_from((a, &env.var_mappings)))
                        .collect::<Result<_, _>>()?;
                    let right = Expected::new(last_pos, &Function { name: f_name.clone(), args });
                    constr.add("function call", &right, &fun_exp);
                }
            } else {
                // Resort to looking up in Context
                let fun = ctx.function(&f_name, ast.pos)?;
                constr = call_parameters(ast, &fun.arguments, &None, args, ctx, &constr)?;
                let fun_ret_exp = Expected::new(ast.pos, &Type { name: fun.ret_ty });
                // entire AST is either fun ret ty or statement
                constr.add(
                    "function call",
                    &Expected::try_from((ast, &env.var_mappings))?,
                    &fun_ret_exp,
                );

                if !fun.raises.is_empty() {
                    if let Some(raises) = &env.raises {
                        let raises_exp = Expected::new(ast.pos, &Raises { name: fun.raises });
                        constr.add("function call", raises, &raises_exp);
                    } else if !constr.is_top_level() {
                        let msg = format!("Exceptions not covered: {}", &fun.raises);
                        return Err(vec![TypeErr::new(ast.pos, &msg)]);
                    }
                }
            }

            Ok((constr, env))
        }
        Node::PropertyCall { instance, property } => {
            property_call(&mut vec![instance.deref().clone()], property, env, ctx, constr)
        }
        Node::Index { item, range } => {
            let (mut constr, _) = generate(range, env, ctx, constr)?;

            let name = Name::from(&HashSet::from([clss::INT, clss::SLICE]));
            constr.add(
                "index range",
                &Expected::new(range.pos, &Expect::Type { name }),
                &Expected::try_from((range, &env.var_mappings))?,
            );

            let (temp_type, env) = env.temp_var();
            let name = Name::from(temp_type.as_str());
            constr.add(
                "index of collection",
                &Expected::new(ast.pos, &Expect::Type { name: name.clone() }),
                &Expected::try_from((ast, &env.var_mappings))?,
            );

            let mut constr = constr_col(item, &env, &mut constr, Some(name))?;
            generate(item, &env, ctx, &mut constr)
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Was expecting call")]),
    }
}

fn check_iden_mut(id: &Identifier, env: &Environment, pos: Position) -> TypeResult<()> {
    let errors: Vec<String> = id
        .fields(pos)?
        .iter()
        .flat_map(|(f_mut, var)| match env.get_var(var) {
            Some(exps) if *f_mut => exps
                .iter()
                .filter(|(is_mut, _)| !*is_mut)
                .map(|(_, var)| format!("Cannot change mutability of '{}' in reassign", var))
                .collect(),
            _ if !f_mut => vec![format!("Cannot change mutability of '{}' in reassign", var)],
            _ => vec![format!("Cannot reassign to undefined '{}'", var)],
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.iter().map(|msg| TypeErr::new(pos, msg)).collect())
    }
}

fn call_parameters(
    self_ast: &AST,
    possible: &[FunctionArg],
    self_arg: &Option<Expect>,
    args: &[AST],
    ctx: &Context,
    constr: &ConstrBuilder,
) -> Result<ConstrBuilder, Vec<TypeErr>> {
    let mut constr = constr.clone();
    let args = if let Some(self_arg) = self_arg {
        let mut new_args = vec![(self_ast.pos, self_arg.clone())];
        new_args.append(
            &mut args.iter().map(|arg| (arg.pos, Expression { ast: arg.clone() })).collect(),
        );
        new_args
    } else {
        args.iter().map(|arg| (arg.pos, Expression { ast: arg.clone() })).collect()
    };

    for either_or_both in possible.iter().zip_longest(args.iter()) {
        match either_or_both {
            Both(fun_arg, (pos, arg)) => {
                let ty = &fun_arg.ty.clone().ok_or_else(|| {
                    TypeErr::new(*pos, "Function argument must have type parameters")
                })?;

                let arg_exp = Expected::new(*pos, arg);
                let name = Name::from(&ctx.class(ty, *pos)?);
                constr.add("call parameters", &arg_exp, &Expected::new(*pos, &Type { name }))
            }
            Left(fun_arg) if !fun_arg.has_default => {
                let pos = Position::new(self_ast.pos.end, self_ast.pos.end);
                let msg = format!("Expected argument: '{}' has no default", fun_arg);
                return Err(vec![TypeErr::new(pos, &msg)]);
            }
            Right((pos, _)) => return Err(vec![TypeErr::new(*pos, "Unexpected argument")]),
            _ => {}
        }
    }

    Ok(constr)
}

fn property_call(
    instance: &mut Vec<AST>,
    property: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let last_inst = instance
        .last()
        .ok_or_else(|| vec![TypeErr::new(property.pos, "Internal error in property call")])?;

    let (mut constr, access) = match &property.node {
        Node::PropertyCall { instance: inner, property } => {
            let (mut constr, _) = property_call(instance, inner, env, ctx, constr)?;

            instance.push(*inner.clone());
            return property_call(instance, property, env, ctx, &mut constr);
        }
        Node::Id { lit } => {
            match &last_inst.node {
                Node::Id { lit: instance } if instance == arg::SELF => {
                    // no objects has attribute self, so if not top-level error elsewhere
                    if env.unassigned.contains(lit) {
                        let msg = format!("Cannot access unassigned field {}", lit);
                        return Err(vec![TypeErr::new(property.pos, &msg)]);
                    }
                }
                _ => {}
            }

            let access_exp = Expected::new(property.pos, &Field { name: lit.clone() });
            (constr.clone(), access_exp)
        }
        Node::FunctionCall { name, args } => {
            let (constr, _) = gen_vec(args, env, ctx, constr)?;
            let args = vec![last_inst.clone()]
                .iter()
                .chain(args)
                .map(|ast| Expected::try_from((ast, &env.var_mappings)))
                .collect::<Result<_, _>>()?;

            let function = Function { name: StringName::try_from(name)?, args };
            (constr, Expected::new(property.pos, &function))
        }

        _ => return Err(vec![TypeErr::new(property.pos, "Expected property call")]),
    };

    let instance_exp: Vec<Expected> = instance
        .iter()
        .map(|ast| Expected::try_from((ast, &env.var_mappings)))
        .collect::<Result<Vec<Expected>, _>>()?;
    let access = instance_exp.iter().rfold(access, |acc, entity| {
        let access = Access { entity: Box::from(entity.clone()), name: Box::from(acc) };
        Expected::new(entity.pos, &access)
    });

    let ast: AST = instance.iter().fold(property.clone(), |acc, ast| {
        let (instance, property) = (Box::from(ast.clone()), Box::from(acc));
        AST::new(ast.pos, Node::PropertyCall { instance, property })
    });
    let ast = Expected::try_from((&ast, &env.var_mappings))?;

    constr.add("call property", &access, &ast);
    Ok((constr.clone(), env.clone()))
}

/// Check if AST is something was can be re-assigned to.
///
/// This is true if it is a valid identifier, or a property call which is a identifier.
/// A property call may not be a tuple, however.
fn check_reassignable(ast: &AST) -> TypeResult<Identifier> {
    match &ast.node {
        Node::PropertyCall { instance, property } => match check_reassignable(property)? {
            Identifier::Multi(_) => {
                let msg = format!("Cannot reassign to {}", &ast.node);
                Err(vec![TypeErr::new(ast.pos, &msg)])
            }
            Identifier::Single(m, prop_call) => {
                let (_, inst_call) = match check_reassignable(instance)? {
                    Identifier::Single(m, call) => (m, call),
                    Identifier::Multi(_) => {
                        let msg = format!("Cannot reassign to {}", &ast.node);
                        return Err(vec![TypeErr::new(ast.pos, &msg)]);
                    }
                };

                let id_call = IdentiCall::Call(Box::from(inst_call), Box::from(prop_call));
                Ok(Identifier::Single(m, id_call))
            }
        },
        _ => Identifier::try_from(ast).map_err(|errs| {
            errs.iter()
                .map(|err| {
                    let msg = format!("Cannot reassign to {}: {}", &ast.node, &err.msg);
                    TypeErr::new(ast.pos, &msg)
                })
                .collect()
        }),
    }
}

fn reassign_op(
    ast: &AST,
    left: &AST,
    right: &AST,
    op: &NodeOp,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    let left = Box::from(left.clone());
    let right = Box::from(right.clone());

    let right = Box::from(AST::new(
        ast.pos,
        match op {
            NodeOp::Add => Node::Add { left: left.clone(), right },
            NodeOp::Sub => Node::Sub { left: left.clone(), right },
            NodeOp::Mul => Node::Mul { left: left.clone(), right },
            NodeOp::Div => Node::Div { left: left.clone(), right },
            NodeOp::Pow => Node::Pow { left: left.clone(), right },
            NodeOp::BLShift => Node::BLShift { left: left.clone(), right },
            NodeOp::BRShift => Node::BRShift { left: left.clone(), right },
            other => {
                let msg = format!("Cannot reassign using operator '{}'", other);
                return Err(vec![TypeErr::new(ast.pos, &msg)]);
            }
        },
    ));

    let (mut constr, env) = generate(&right, env, ctx, constr)?;

    let node = Node::Reassign { left, right, op: NodeOp::Assign };
    let simple_assign_ast = AST::new(ast.pos, node);
    generate(&simple_assign_ast, &env, ctx, &mut constr)
}
