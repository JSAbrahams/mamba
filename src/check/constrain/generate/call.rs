use std::cmp::Ordering;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::generate::{Constrained, gen_vec, generate};
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::statement::check_raises_caught;
use crate::check::context::{arg, Context, function, LookupClass, LookupFunction};
use crate::check::context::arg::FunctionArg;
use crate::check::context::arg::python::SELF;
use crate::check::context::function::python::GET_ITEM;
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
            check_iden_mut(&identifier, env, constr, left.pos)?;

            if let NodeOp::Assign = op {
                let env_assigned_to: Environment = identifier
                    .all_calls()
                    .iter()
                    .flat_map(|call| call.without_obj(arg::SELF, left.pos))
                    .flat_map(|identi_call| match identi_call {
                        IdentiCall::Iden(var) => Some(var),
                        _ => None,
                    })
                    .fold(env.clone(), |env, self_var| env.assigned_to(&self_var));

                constr.add("reassign", &Expected::from(left), &Expected::from(right), env);
                generate(right, &env_assigned_to, ctx, constr)?;
                generate(left, &env_assigned_to, ctx, constr)?;
                Ok(env_assigned_to)
            } else {
                reassign_op(ast, left, right, op, env, ctx, constr)
            }
        }
        Node::FunctionCall { name, args } => {
            let f_name = StringName::try_from(name)?;
            gen_vec(args, env, false, ctx, constr)?;

            Ok(if f_name == StringName::from(function::PRINT) {
                args.iter()
                    .map(|arg| Constraint::stringy("print", &Expected::from(arg)))
                    .for_each(|cons| constr.add_constr(&cons, env));

                let name = Name::empty();
                constr.add("print", &Expected::new(ast.pos, &Type { name }), &Expected::from(ast), env);
                env.clone()
            } else if let Some(functions) = env.get_var(&f_name.name, &constr.var_mapping) {
                if !f_name.generics.is_empty() {
                    let msg = "Anonymous function call cannot have generics";
                    return Err(vec![TypeErr::new(name.pos, msg)]);
                }

                for (_, fun_exp) in functions {
                    let last_pos = args.last().map_or_else(|| name.pos, |a| a.pos);
                    let args = args.iter().map(Expected::from).collect();
                    let right = Expected::new(last_pos, &Function { name: f_name.clone(), args });
                    constr.add("function call", &right, &fun_exp, env);
                }
                env.clone()
            } else {
                // Resort to looking up in Context
                let fun = ctx.function(&f_name, ast.pos)?;
                call_parameters(ast, &fun.arguments, &None, args, ctx, env, constr)?;
                let fun_ret_exp = Expected::new(ast.pos, &Type { name: fun.ret_ty });
                // entire AST is either fun ret ty or statement
                constr.add("function call", &Expected::from(ast), &fun_ret_exp, env);

                check_raises_caught(&fun.raises.names, env, ctx, ast.pos)?;
                env.clone()
            })
        }
        Node::PropertyCall { instance, property } => {
            property_call(&mut vec![instance.deref().clone()], property, env, ctx, constr)
        }
        Node::Index { item, range } => {
            let getitem_ast = AST::new(ast.pos, Node::PropertyCall {
                instance: item.clone(),
                property: Box::from(AST::new(ast.pos, Node::FunctionCall {
                    name: Box::new(AST::new(range.pos, Node::Id { lit: String::from(GET_ITEM) })),
                    args: vec![*range.clone()],
                })),
            });
            let (existing_ast, new_ast) = (Expected::from(ast), Expected::from(&getitem_ast));
            constr.add("index", &new_ast, &existing_ast, env);

            generate(&getitem_ast, env, ctx, constr)
        }

        _ => Err(vec![TypeErr::new(ast.pos, "Was expecting call")]),
    }
}

fn check_iden_mut(id: &Identifier, env: &Environment, constr: &mut ConstrBuilder, pos: Position) -> TypeResult<()> {
    let errors: Vec<String> = id
        .fields(pos)?
        .iter()
        .flat_map(|(f_mut, var)| match env.get_var(var, &constr.var_mapping) {
            Some(exps) if *f_mut => exps
                .iter()
                .filter(|(is_mut, _)| !*is_mut)
                .map(|(_, var)| format!("Cannot change mutability of '{var}' in reassign"))
                .collect(),
            _ if !f_mut => vec![format!("Cannot change mutability of '{var}' in reassign")],
            _ if var == SELF && env.class.is_some() => vec![],
            _ => vec![format!("Cannot reassign to undefined '{var}'")]
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
    env: &Environment,
    constr: &mut ConstrBuilder,
) -> Constrained<()> {
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
                constr.add("call parameters", &Expected::new(*pos, &Type { name }), &arg_exp, env)
            }
            Left(fun_arg) if !fun_arg.has_default => {
                let pos = Position::new(self_ast.pos.end, self_ast.pos.end);
                let msg = format!("Expected argument: '{fun_arg}' has no default");
                return Err(vec![TypeErr::new(pos, &msg)]);
            }
            Right((pos, _)) => return Err(vec![TypeErr::new(*pos, "Unexpected argument")]),
            _ => {}
        }
    }

    Ok(())
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

    let access = match &property.node {
        Node::PropertyCall { instance: inner, property } => {
            property_call(instance, inner, env, ctx, constr)?;
            instance.push(*inner.clone());
            return property_call(instance, property, env, ctx, constr);
        }
        Node::Id { lit } => {
            if let Node::Id { lit: instance } = &last_inst.node {
                if instance == arg::SELF && env.unassigned.contains(lit) {
                    let msg = format!("Cannot access unassigned field {lit}");
                    return Err(vec![TypeErr::new(property.pos, &msg)]);
                }
            }

            Expected::new(property.pos, &Field { name: lit.clone() })
        }
        Node::FunctionCall { name, args } => {
            gen_vec(args, env, false, ctx, constr)?;
            let args = vec![last_inst.clone()].iter().chain(args).map(Expected::from).collect();
            let function = Function { name: StringName::try_from(name)?, args };
            Expected::new(property.pos, &function)
        }

        _ => return Err(vec![TypeErr::new(property.pos, "Expected property call")]),
    };

    let entire_call_as_ast: AST = instance.iter().rfold(property.clone(), |acc, ast| {
        let (instance, property) = (Box::from(ast.clone()), Box::from(acc));
        AST::new(ast.pos, Node::PropertyCall { instance, property })
    });
    let entire_call_as_ast = Expected::from(&entire_call_as_ast);

    let ast_without_access = match instance.len().cmp(&1) {
        Ordering::Less => panic!("Internal error in access"),
        Ordering::Equal => last_inst.clone(),
        Ordering::Greater => {
            let last = instance.remove(instance.len() - 1);
            instance.iter().rfold(last, |acc, ast| {
                let (instance, property) = (Box::from(ast.clone()), Box::from(acc));
                AST::new(ast.pos, Node::PropertyCall { instance, property })
            })
        }
    };

    let entity = Box::new(Expected::from(&ast_without_access));
    let msg = format!("access property of {entity}");
    let access = Expected::new(
        ast_without_access.pos.union(access.pos),
        &Access { entity, name: Box::new(access) },
    );
    constr.add(&msg, &access, &entire_call_as_ast, env);

    generate(&ast_without_access, env, ctx, constr)?;
    Ok(env.clone())
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
    let (left, right) = (Box::from(left.clone()), Box::from(right.clone()));
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
                let msg = format!("Cannot reassign using operator '{other}'");
                return Err(vec![TypeErr::new(ast.pos, &msg)]);
            }
        },
    ));

    generate(&right, env, ctx, constr)?;

    let node = Node::Reassign { left, right, op: NodeOp::Assign };
    let simple_assign_ast = AST::new(ast.pos, node);
    generate(&simple_assign_ast, env, ctx, constr)?;

    Ok(env.clone())
}
