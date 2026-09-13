use std::cmp::Ordering;
use std::convert::TryFrom;
use std::ops::Deref;

use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;

use crate::check::constrain::constraint::builder::ConstrBuilder;
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::expected::{Expect, Expected};
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::generate::env::Environment;
use crate::check::constrain::generate::operation::gen_magic;
use crate::check::constrain::generate::statement::check_raises_caught;
use crate::check::constrain::generate::{gen_vec, generate, Constrained};
use crate::check::context::arg::python::SELF;
use crate::check::context::arg::FunctionArg;
use crate::check::context::clss::{GetField, GetFun};
use crate::check::context::function::python::{GET_ITEM, SET_ITEM};
use crate::check::context::{arg, function, Context, LookupClass, LookupFunction};
use crate::check::ident::{IdentiCall, Identifier};
use crate::check::name::string_name::StringName;
use crate::check::name::{Empty, Name};
use crate::check::result::{TypeErr, TypeResult};
use crate::check::NEW;
use crate::common::position::Position;
use crate::parse::ast::node_op::NodeOp;
use crate::parse::ast::{Node, AST};

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
            check_pure_assign(&identifier, env, left.pos)?;

            if let NodeOp::Assign = op {
                if let Node::Index { item, range } = &left.node {
                    gen_set_item(item, range, right, env, ctx, constr)?;
                } else if let Node::FunctionCall { name, args } = &left.node {
                    // Round-bracket equivalent of `Node::Index`: `item(range) := right`.
                    let range = args.first().ok_or_else(|| {
                        vec![TypeErr::new(
                            left.pos,
                            "Cannot reassign to a call with no arguments",
                        )]
                    })?;
                    gen_set_item(name, range, right, env, ctx, constr)?;
                } else {
                    constr.add(
                        "reassign",
                        &Expected::from(left),
                        &Expected::from(right),
                        env,
                    );
                    generate(right, env, ctx, constr)?;
                    generate(left, env, ctx, constr)?;
                }
                Ok(env.clone())
            } else {
                reassign_op(ast, left, right, op, env, ctx, constr)
            }
        }
        Node::FunctionCall { name, args } => {
            let f_name = StringName::try_from(name)?;
            gen_vec(args, env, false, ctx, constr)?;

            Ok(if f_name == StringName::from(function::PRINT) {
                if env.in_pure {
                    let msg = format!(
                        "A pure function cannot call '{}', which writes to standard output",
                        function::PRINT
                    );
                    return Err(vec![TypeErr::new(ast.pos, &msg)]);
                }
                args.iter()
                    .map(|arg| Constraint::stringy("print", &Expected::from(arg)))
                    .for_each(|cons| constr.add_constr(&cons, env));

                let name = Name::empty();
                constr.add(
                    "print",
                    &Expected::new(ast.pos, &Type { name }),
                    &Expected::from(ast),
                    env,
                );
                env.clone()
            } else if let Some(functions) = env.get_var(&f_name.name, &constr.var_mapping) {
                if !f_name.generics.is_empty() {
                    let msg = "Anonymous function call cannot have generics";
                    return Err(vec![TypeErr::new(name.pos, msg)]);
                }

                for (_, fun_exp) in functions {
                    let call_args = args.iter().map(Expected::from).collect();
                    let access = Expected::new(
                        ast.pos,
                        &Access {
                            entity: Box::new(fun_exp.clone()),
                            name: Box::new(Expected::new(ast.pos, &Call { args: call_args })),
                        },
                    );
                    constr.add("function call", &Expected::from(ast), &access, env);
                }
                env.clone()
            } else {
                // Resort to looking up in Context
                check_constructor_in_class(&f_name, env, ctx, ast.pos)?;
                let fun = ctx.function(&f_name, ast.pos)?;
                check_pure_call(&f_name, fun.pure, env, ast.pos)?;
                call_parameters(ast, &fun.arguments, &None, args, ctx, env, constr)?;
                let fun_ret_exp = Expected::new(ast.pos, &Type { name: fun.ret_ty });
                // entire AST is either fun ret ty or statement
                constr.add("function call", &Expected::from(ast), &fun_ret_exp, env);

                check_raises_caught(&fun.raises.names, env, ctx, ast.pos)?;
                env.clone()
            })
        }
        Node::PropertyCall { instance, property } => {
            if let Some(env) = gen_associated_call(ast, instance, property, env, ctx, constr)? {
                return Ok(env);
            }
            property_call(
                &mut vec![instance.deref().clone()],
                property,
                env,
                ctx,
                constr,
            )
        }
        Node::Index { item, range } => gen_magic(GET_ITEM, ast, item, range, env, ctx, constr),

        _ => Err(vec![TypeErr::new(ast.pos, "Was expecting call")]),
    }
}

/// A pure function may only call other pure functions.
///
/// Calling an impure one would let its side effects, or its dependence on state outside its
/// arguments, leak into a function that claims to have neither.
fn check_pure_call(
    name: &StringName,
    callee_pure: bool,
    env: &Environment,
    pos: Position,
) -> TypeResult<()> {
    if env.in_pure && !callee_pure {
        let msg = format!("A pure function cannot call '{name}', which is not pure");
        Err(vec![TypeErr::new(pos, &msg)])
    } else {
        Ok(())
    }
}

/// Applying a class to its arguments is the construction primitive, and it is private.
///
/// It is only in scope inside that class's own body, which is what lets an explicit `new`
/// enforce an invariant: outside, there is no way around it. Everywhere else, construction
/// goes through `new`.
fn check_constructor_in_class(
    f_name: &StringName,
    env: &Environment,
    ctx: &Context,
    pos: Position,
) -> TypeResult<()> {
    // A real function of that name wins, exactly as it does in the context lookup.
    let is_constructor = !ctx.functions.iter().any(|f| f.name == *f_name)
        && ctx.classes.iter().any(|c| c.name == *f_name);
    if !is_constructor || env.class.as_ref() == Some(f_name) {
        return Ok(());
    }

    let msg = format!(
        "Cannot construct '{f_name}' here. '{f_name}(..)' is only in scope within '{f_name}' itself, so use '{f_name}.new(..)'"
    );
    Err(vec![TypeErr::new(pos, &msg)])
}

/// A call on a class name rather than on a value, such as `Point.new(1, 2)`.
///
/// The class acts as a namespace, and the function it names takes no `self`. Both the context
/// and the Python backend already model such a function, so only resolving the receiver is
/// needed here.
///
/// [None] when the receiver is not a class name, leaving it to be handled as an ordinary
/// property access on a value.
fn gen_associated_call(
    ast: &AST,
    instance: &AST,
    property: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> TypeResult<Option<Environment>> {
    let lit = match &instance.node {
        Node::Id { lit } => lit,
        _ => return Ok(None),
    };
    // A variable of the same name shadows the class.
    if env.get_var(lit, &constr.var_mapping).is_some() {
        return Ok(None);
    }
    let class = match ctx.class(&StringName::from(lit.as_str()), instance.pos) {
        Ok(class) => class,
        Err(_) => return Ok(None),
    };
    let (name, args) = match &property.node {
        Node::FunctionCall { name, args } => (name, args),
        _ => return Ok(None),
    };

    let f_name = StringName::try_from(name)?;
    // Every class gets a `new` taking its arguments, unless it declares one of its own.
    let fun = match class.fun(&f_name, property.pos) {
        Ok(fun) => fun,
        Err(_) if f_name.name.as_str() == NEW => class.constructor(true),
        Err(err) => return Err(err),
    };
    check_pure_call(&f_name, fun.pure, env, ast.pos)?;

    gen_vec(args, env, false, ctx, constr)?;
    call_parameters(ast, &fun.arguments, &None, args, ctx, env, constr)?;

    let ret = Expected::new(ast.pos, &Type { name: fun.ret_ty });
    constr.add("associated function call", &Expected::from(ast), &ret, env);
    check_raises_caught(&fun.raises.names, env, ctx, ast.pos)?;
    Ok(Some(env.clone()))
}

/// The receiver of a property access, and the classes it may be, where the rules apply to it.
///
/// [None] when the rules do not reach this receiver: it is not a plain identifier, it was
/// defined by the pure function's own body (so it is destroyed on exit and is fair game), or
/// its type is not known here. Generation runs before unification, so only `self` and an
/// annotated variable or argument resolve.
fn pure_receiver<'a>(
    instance: &'a AST,
    env: &Environment,
    constr: &ConstrBuilder,
) -> Option<(&'a String, Vec<StringName>)> {
    let lit = match &instance.node {
        Node::Id { lit } => lit,
        _ => return None,
    };

    let classes: Vec<StringName> = if lit == arg::SELF {
        env.class.iter().cloned().collect()
    } else if env.pure_nonlocal.contains(lit) {
        env.get_var(lit, &constr.var_mapping)
            .into_iter()
            .flatten()
            .filter_map(|(_, exp)| match exp.expect {
                Type { name } => Some(name.as_direct()),
                _ => None,
            })
            .flatten()
            .collect()
    } else {
        return None;
    };

    Some((lit, classes))
}

/// A pure function may only read fields that are not `mut`.
///
/// A `mut` field can change between two calls with the same arguments, so reading one would
/// break the guarantee that the result depends on the arguments alone.
fn check_pure_field_read(
    instance: &AST,
    field: &str,
    env: &Environment,
    ctx: &Context,
    constr: &ConstrBuilder,
    pos: Position,
) -> TypeResult<()> {
    if !env.in_pure {
        return Ok(());
    }
    let (lit, classes) = match pure_receiver(instance, env, constr) {
        Some(receiver) => receiver,
        None => return Ok(()),
    };

    for class in classes {
        // An unresolvable class or field is left to the rest of the checker to report.
        if let Ok(class) = ctx.class(&class, pos) {
            if let Ok(f) = class.field(field, pos) {
                if f.mutable {
                    let msg = format!(
                        "A pure function cannot read '{field}' of '{lit}', as it is declared 'mut'"
                    );
                    return Err(vec![TypeErr::new(pos, &msg)]);
                }
            }
        }
    }
    Ok(())
}

/// A pure function may only call pure methods on a receiver it did not itself define.
///
/// The receiver's class has to be known here, since generation runs before unification. That
/// covers `self`, and any variable or argument with an annotated type, which is exactly where
/// the rule matters: a receiver the function built itself is fair game either way.
fn check_pure_method(
    instance: &AST,
    name: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &ConstrBuilder,
    pos: Position,
) -> TypeResult<()> {
    if !env.in_pure {
        return Ok(());
    }
    let (lit, classes) = match pure_receiver(instance, env, constr) {
        Some(receiver) => receiver,
        None => return Ok(()),
    };

    let f_name = StringName::try_from(name)?;
    for class in classes {
        // An unresolvable class is left to the rest of the checker to report.
        if let Ok(class) = ctx.class(&class, pos) {
            if let Ok(fun) = class.fun(&f_name, pos) {
                if !fun.pure {
                    let msg = format!(
                        "A pure function cannot call '{f_name}' on '{lit}', as it is not pure"
                    );
                    return Err(vec![TypeErr::new(pos, &msg)]);
                }
            }
        }
    }
    Ok(())
}

/// A pure function may not assign through anything it did not itself define.
///
/// Assigning to an argument only rebinds a local name, which is fine. Assigning to a *field*
/// of one reaches through to the caller's value, and assigning to a mutable variable from an
/// enclosing scope leaks out of the function altogether. Both hold for `self` too, which is
/// just another argument.
fn check_pure_assign(id: &Identifier, env: &Environment, pos: Position) -> TypeResult<()> {
    if !env.in_pure {
        return Ok(());
    }

    let errors: Vec<String> = id
        .all_calls()
        .iter()
        .filter_map(|call| match call {
            IdentiCall::Iden(var) if env.outer_mut.contains(var) => Some(format!(
                "A pure function cannot assign to '{var}', which is defined outside it"
            )),
            IdentiCall::Iden(_) => None,
            call => {
                let object = call.object(pos).ok()?;
                if env.pure_nonlocal.contains(&object) {
                    Some(format!(
                        "A pure function cannot assign to a field of '{object}', as that would modify the caller's value"
                    ))
                } else {
                    None
                }
            }
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.iter().map(|msg| TypeErr::new(pos, msg)).collect())
    }
}

fn check_iden_mut(
    id: &Identifier,
    env: &Environment,
    constr: &mut ConstrBuilder,
    pos: Position,
) -> TypeResult<()> {
    let errors: Vec<String> = id
        .fields(pos)?
        .iter()
        .flat_map(|(f_mut, var)| match env.get_var(var, &constr.var_mapping) {
            Some(exps) if *f_mut => exps
                .iter()
                .filter(|(is_mut, _)| !*is_mut)
                .map(|(_, var)| format!("Cannot assign to '{var}', which was not declared 'mut'"))
                .collect(),
            _ if !f_mut => vec![format!(
                "Cannot assign to '{var}', which was not declared 'mut'"
            )],
            _ if var == SELF && env.class.is_some() => vec![],
            _ => vec![format!("Cannot reassign to undefined '{var}'")],
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
            &mut args
                .iter()
                .map(|arg| (arg.pos, Expression { ast: arg.clone() }))
                .collect(),
        );
        new_args
    } else {
        args.iter()
            .map(|arg| (arg.pos, Expression { ast: arg.clone() }))
            .collect()
    };

    for either_or_both in possible.iter().zip_longest(args.iter()) {
        match either_or_both {
            Both(fun_arg, (pos, arg)) => {
                let ty = &fun_arg.ty.clone().ok_or_else(|| {
                    TypeErr::new(*pos, "Function argument must have type parameters")
                })?;

                let arg_exp = Expected::new(*pos, arg);
                let name = Name::from(&ctx.class(ty, *pos)?);
                constr.add(
                    "call parameters",
                    &Expected::new(*pos, &Type { name }),
                    &arg_exp,
                    env,
                )
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
    let last_inst = instance.last().ok_or_else(|| {
        vec![TypeErr::new(
            property.pos,
            "Internal error in property call",
        )]
    })?;

    let access = match &property.node {
        Node::PropertyCall {
            instance: inner,
            property,
        } => {
            property_call(instance, inner, env, ctx, constr)?;
            instance.push(*inner.clone());
            return property_call(instance, property, env, ctx, constr);
        }
        Node::Id { lit } => {
            check_pure_field_read(last_inst, lit, env, ctx, constr, property.pos)?;
            Expected::new(property.pos, &Field { name: lit.clone() })
        }
        Node::FunctionCall { name, args } => {
            check_pure_method(last_inst, name, env, ctx, constr, property.pos)?;
            gen_vec(args, env, false, ctx, constr)?;
            let args = [last_inst.clone()]
                .iter()
                .chain(args)
                .map(Expected::from)
                .collect();
            let function = Function {
                name: StringName::try_from(name)?,
                args,
            };
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
        &Access {
            entity,
            name: Box::new(access),
        },
    );
    constr.add(&msg, &access, &entire_call_as_ast, env);

    generate(&ast_without_access, env, ctx, constr)?;
    Ok(env.clone())
}

/// Generate the constraint for `item(range) := value`, i.e. `item.__setitem__(range, value)`.
///
/// Deliberately not [gen_magic], which is built for binary magic methods (`self`, one other
/// argument); `__setitem__` takes two logical arguments (`range`, `value`) besides `self`.
fn gen_set_item(
    item: &AST,
    range: &AST,
    value: &AST,
    env: &Environment,
    ctx: &Context,
    constr: &mut ConstrBuilder,
) -> Constrained {
    gen_vec(
        &[item.clone(), range.clone(), value.clone()],
        env,
        env.is_def_mode,
        ctx,
        constr,
    )?;

    let access = Expected::new(
        item.pos,
        &Access {
            entity: Box::new(Expected::from(item)),
            name: Box::new(Expected::new(
                item.pos,
                &Function {
                    name: StringName::from(SET_ITEM),
                    args: vec![
                        Expected::from(item),
                        Expected::from(range),
                        Expected::from(value),
                    ],
                },
            )),
        },
    );
    constr.add("set item", &Expected::any(item.pos), &access, env);
    Ok(env.clone())
}

/// Check if AST is something was can be re-assigned to.
///
/// This is true if it is a valid identifier, a property call which is a identifier, or an index (`item(range)`) into a mutable collection.
/// A property call or index may not be a tuple, however.
fn check_reassignable(ast: &AST) -> TypeResult<Identifier> {
    match &ast.node {
        Node::PropertyCall { instance, property } => match check_reassignable(property)? {
            Identifier::Multi(_) => {
                let msg = format!("Cannot reassign to {}", ast.node);
                Err(vec![TypeErr::new(ast.pos, &msg)])
            }
            Identifier::Single(m, prop_call) => {
                let (_, inst_call) = match check_reassignable(instance)? {
                    Identifier::Single(m, call) => (m, call),
                    Identifier::Multi(_) => {
                        let msg = format!("Cannot reassign to {}", ast.node);
                        return Err(vec![TypeErr::new(ast.pos, &msg)]);
                    }
                };

                let id_call = IdentiCall::Call(Box::from(inst_call), Box::from(prop_call));
                Ok(Identifier::Single(m, id_call))
            }
        },
        Node::Index { item, .. } => match check_reassignable(item)? {
            Identifier::Multi(_) => {
                let msg = format!("Cannot reassign to {}", ast.node);
                Err(vec![TypeErr::new(ast.pos, &msg)])
            }
            single => Ok(single),
        },
        Node::FunctionCall { name, args } if args.len() == 1 => match check_reassignable(name)? {
            Identifier::Multi(_) => {
                let msg = format!("Cannot reassign to {}", ast.node);
                Err(vec![TypeErr::new(ast.pos, &msg)])
            }
            single => Ok(single),
        },
        _ => Identifier::try_from(ast).map_err(|errs| {
            errs.iter()
                .map(|err| {
                    let msg = format!("Cannot reassign to {}: {}", ast.node, err.msg);
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
            NodeOp::Add => Node::Add {
                left: left.clone(),
                right,
            },
            NodeOp::Sub => Node::Sub {
                left: left.clone(),
                right,
            },
            NodeOp::Mul => Node::Mul {
                left: left.clone(),
                right,
            },
            NodeOp::Div => Node::Div {
                left: left.clone(),
                right,
            },
            NodeOp::Pow => Node::Pow {
                left: left.clone(),
                right,
            },
            other => {
                let msg = format!("Cannot reassign using operator '{other}'");
                return Err(vec![TypeErr::new(ast.pos, &msg)]);
            }
        },
    ));

    generate(&right, env, ctx, constr)?;

    let node = Node::Reassign {
        left,
        right,
        op: NodeOp::Assign,
    };
    let simple_assign_ast = AST::new(ast.pos, node);
    generate(&simple_assign_ast, env, ctx, constr)?;

    Ok(env.clone())
}
