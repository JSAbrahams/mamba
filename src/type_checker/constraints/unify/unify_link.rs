use std::collections::HashSet;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::unify::substitute::substitute;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::Context;
use crate::type_checker::ty_name::actual::ActualTypeName;
use crate::type_checker::ty_name::TypeName;
use crate::type_checker::util::comma_delimited;

/// Unifies all constraints.

/// We use a mutable reference to constraints for performance reasons.
/// Otherwise, we have to make a entirely new copy of the list of all
/// constraints each time we do a recursive call to unify link.
pub fn unify_link(
    constr: &mut Constraints,
    sub: &Constraints,
    ctx: &Context,
    total: usize
) -> Unified {
    if let Some(constraint) = &constr.pop_constr() {
        let (left, right) = (constraint.parent.clone(), constraint.child.clone());
        println!(
            "{:width$} [unifying {}\\{}{}{}] {} = {}",
            format!("({}-{})", left.pos.start, right.pos.start),
            total - constr.len(),
            total,
            if constraint.flagged { " (fl)" } else { "" },
            if constraint.substituted { " (sub)" } else { "" },
            left.expect,
            right.expect,
            width = 15
        );

        match (&left.expect, &right.expect) {
            // trivially equal
            (l_expect, r_expect) if l_expect.structurally_eq(r_expect) => {
                let mut constr = substitute(&left, &right, constr, &left.pos)?;
                unify_link(&mut constr, sub, ctx, total)
            }

            (Expression { ast }, ExpressionAny) | (ExpressionAny, Expression { ast }) =>
                if ast.node.is_expression() {
                    unify_link(constr, sub, ctx, total)
                } else {
                    Err(vec![TypeErr::new(
                        &ast.pos,
                        &format!("Expected expression but was {}", ast.node.name())
                    )])
                },

            (Type { .. }, ExpressionAny) | (ExpressionAny, Type { .. }) =>
                unify_link(constr, sub, ctx, total),
            (Truthy, ExpressionAny) | (ExpressionAny, Truthy) =>
                unify_link(constr, sub, ctx, total),

            (Expression { ast: AST { node: Node::Undefined, .. } }, Type { type_name })
                if !type_name.is_nullable() =>
                Err(vec![TypeErr::new(
                    &left.pos,
                    &format!("Expected {} but was {}", type_name.as_nullable(), type_name)
                )]),
            (Type { type_name }, Expression { ast: AST { node: Node::Undefined, .. } })
                if !type_name.is_nullable() =>
                Err(vec![TypeErr::new(
                    &right.pos,
                    &format!("Expected {} but was {}", type_name.as_nullable(), type_name)
                )]),

            (Expression { .. }, Expression { .. })
            | (Expression { .. }, Type { .. })
            | (Expression { .. }, Collection { .. })
            | (Expression { .. }, Truthy) => {
                let mut sub = Constraints::new(&vec![constraint.clone()], &constr.in_class);
                sub.append(&substitute(&left, &right, &sub, &left.pos)?);

                let mut constr = substitute(&left, &right, &constr, &right.pos)?;
                unify_link(&mut constr, &sub, ctx, total)
            }
            (Collection { .. }, Expression { ast })
            | (Type { .. }, Expression { ast })
            | (Truthy, Expression { ast }) =>
                if ast.node.is_expression() {
                    let mut sub = Constraints::new(&vec![constraint.clone()], &constr.in_class);
                    sub.append(&substitute(&left, &right, &sub, &left.pos)?);

                    let mut constr = substitute(&left, &right, &constr, &left.pos)?;
                    unify_link(&mut constr, &sub, ctx, total)
                } else {
                    Err(vec![TypeErr::new(
                        &ast.pos,
                        &format!("Expected expression but was {}", ast.node.name())
                    )])
                },

            (Type { type_name }, Truthy) | (Truthy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                expr_ty.function(&TypeName::from(function::python::TRUTHY), &left.pos)?;
                unify_link(constr, sub, ctx, total)
            }
            (Type { type_name: l_ty }, Type { type_name: r_ty }) => {
                if l_ty.is_superset(r_ty) {
                    ctx.lookup(l_ty, &left.pos)?;
                    unify_link(constr, sub, ctx, total)
                } else {
                    // TODO construct error based on type of constraint
                    let msg = format!("Expected a {} but got {}", l_ty, r_ty);
                    Err(vec![TypeErr::new(&left.pos, &msg)])
                }
            }

            (Type { type_name }, Implements { type_name: f_name, args })
            | (Implements { type_name: f_name, args }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                let f_name = if f_name == type_name {
                    ctx.lookup(f_name, &left.pos)?;
                    TypeName::from(function::concrete::INIT)
                } else {
                    f_name.clone()
                };

                let possible = expr_ty.function(&f_name, &left.pos)?;
                for function in &possible {
                    if function.private {
                        check_if_parent(
                            &function.name,
                            &constr.in_class,
                            &type_name,
                            ctx,
                            &right.pos
                        )?;
                    }
                }

                let possible_args: HashSet<Vec<FunctionArg>> =
                    possible.iter().map(|f| f.arguments.clone()).collect();
                let (mut constr, added) = unify_fun_arg(&possible_args, &args, constr, &right.pos)?;
                unify_link(&mut constr, &sub, ctx, total + added)
            }

            (
                Implements { type_name: l_name, args: l_args },
                Implements { type_name: r_name, args: r_args }
            ) =>
                if l_name == r_name {
                    let mut added = 0;
                    for pair in l_args.iter().zip_longest(r_args) {
                        match pair {
                            EitherOrBoth::Both(l, r) => {
                                added += 1;
                                constr.eager_push(l, r)
                            }
                            EitherOrBoth::Left(l) =>
                                return Err(vec![TypeErr::new(&l.pos, "Unexpected argument")]),
                            EitherOrBoth::Right(r) =>
                                return Err(vec![TypeErr::new(&r.pos, "Unexpected argument")]),
                        }
                    }

                    unify_link(constr, sub, ctx, total + added)
                } else {
                    Err(vec![TypeErr::new(
                        &left.pos,
                        &format!("{} not equal to {}", left.expect, right.expect)
                    )])
                },

            (Type { type_name }, Raises { raises }) =>
                if raises.contains(type_name) {
                    unify_link(constr, sub, ctx, total)
                } else {
                    let msg = format!(
                        "Unexpected raises {}, must be one of: {}",
                        type_name,
                        comma_delimited(raises)
                    );
                    Err(vec![TypeErr::new(&left.pos, &msg)])
                },
            (Raises { raises }, Type { type_name }) =>
                if raises.contains(type_name) {
                    unify_link(constr, sub, ctx, total)
                } else {
                    let msg = format!(
                        "Unexpected raises {}, must be one of: {}",
                        type_name,
                        comma_delimited(raises)
                    );
                    Err(vec![TypeErr::new(&right.pos, &msg)])
                },

            (Type { type_name }, Nullable) | (Nullable, Type { type_name }) =>
                if type_name.is_nullable() {
                    unify_link(constr, sub, ctx, total)
                } else {
                    Err(vec![TypeErr::new(
                        &left.pos,
                        &format!("Expected {} but was {}", type_name.as_nullable(), type_name)
                    )])
                },

            (Expression { ast: AST { node: Node::Undefined, .. } }, Nullable)
            | (Nullable, Expression { ast: AST { node: Node::Undefined, .. } }) =>
                unify_link(constr, sub, ctx, total),

            (Type { type_name }, Function { args, .. })
            | (Function { args, .. }, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                let functions = expr_ty.anon_fun_params(&left.pos)?;

                let mut count = 0;
                for (f_args, _) in &functions {
                    for possible in f_args.iter().zip_longest(args.iter()) {
                        match possible {
                            EitherOrBoth::Both(type_name, expected) => {
                                count += 1;
                                let ty = Type { type_name: type_name.clone() };
                                let right = Expected::new(&left.pos, &ty);
                                constr.eager_push(expected, &right);
                            }
                            EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => {
                                let msg = format!(
                                    "{} arguments given to function which takes {} arguments",
                                    args.len(),
                                    f_args.len()
                                );
                                return Err(vec![TypeErr::new(&left.pos, &msg)]);
                            }
                        }
                    }
                }

                unify_link(constr, sub, ctx, total + count)
            }

            (Type { type_name }, HasField { name }) | (HasField { name }, Type { type_name }) => {
                let field = ctx.lookup(type_name, &right.pos)?.field(name, &left.pos)?;
                if field.private {
                    let name = ActualTypeName::new(&field.name, &[]);
                    check_if_parent(&name, &constr.in_class, type_name, ctx, &left.pos)?;
                }
                unify_link(constr, sub, ctx, total)
            }

            (Type { type_name }, Collection { ty }) | (Collection { ty }, Type { type_name }) => {
                check_iter(type_name, ty, ctx, constr, &left.pos)?;
                unify_link(constr, sub, ctx, total)
            }
            (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
                constr.push(&Expected::new(&left.pos, l_ty), &Expected::new(&right.pos, r_ty));
                unify_link(constr, sub, ctx, total + 1)
            }

            _ => {
                let pos = format!("({}-{})", left.pos.start, right.pos.start);
                let count = format!("[reinserting {}\\{}]", total - constr.len(), total);
                println!("{:width$} {} {} = {}", pos, count, left.expect, right.expect, width = 17);

                // Defer to later point
                constr.reinsert(&constraint)?;
                unify_link(constr, sub, ctx, total + 1)
            }
        }
    } else {
        Ok(constr.clone())
    }
}

fn unify_fun_arg(
    possible: &HashSet<Vec<FunctionArg>>,
    args: &[Expected],
    constr: &Constraints,
    pos: &Position
) -> Unified<(Constraints, usize)> {
    let mut constr = constr.clone();
    let mut added = 0;

    for f_args in possible {
        for either_or_both in f_args.iter().zip_longest(args.iter()) {
            match either_or_both {
                EitherOrBoth::Both(fun_arg, expected) => {
                    let ty = &fun_arg.ty.as_ref();
                    let type_name = ty
                        .ok_or_else(|| {
                            TypeErr::new(
                                &expected.pos,
                                "Function argument must have type parameters"
                            )
                        })?
                        .clone();

                    added += 1;
                    constr.push(&expected, &Expected::new(&expected.pos, &Type { type_name }))
                }
                EitherOrBoth::Left(fun_arg) if !fun_arg.has_default =>
                    return Err(vec![TypeErr::new(
                        &pos,
                        &format!("Expected argument: expected {}", fun_arg)
                    )]),
                EitherOrBoth::Right(_) =>
                    return Err(vec![TypeErr::new(
                        &pos,
                        &format!(
                            "Unexpected argument, function takes only {} arguments",
                            f_args.len()
                        )
                    )]),
                _ => {}
            }
        }
    }

    Ok((constr, added))
}

fn check_iter(
    type_name: &TypeName,
    ty: &Expect,
    ctx: &Context,
    constr: &mut Constraints,
    pos: &Position
) -> TypeResult<Constraints> {
    let f_name = TypeName::from(function::concrete::ITER);
    for fun in ctx.lookup(type_name, pos)?.function(&f_name, pos)? {
        let msg = format!("{} __iter__ type undefined", type_name);
        let f_ret_ty = fun.ty().ok_or_else(|| TypeErr::new(&pos, &msg))?;

        constr.push(
            &Expected::new(&pos, &Type { type_name: type_name.clone() }),
            &Expected::new(&pos, &Type { type_name: f_ret_ty.clone() })
        );
        constr.push(&Expected::new(&pos, &ty), &Expected::new(&pos, &Type { type_name: f_ret_ty }));
    }

    Ok(constr.clone())
}

fn check_if_parent(
    name: &ActualTypeName,
    in_class: &Vec<TypeName>,
    type_name: &TypeName,
    ctx: &Context,
    pos: &Position
) -> TypeResult<()> {
    let mut in_a_parent = false;
    for class in in_class {
        let is_parent = ctx.lookup(&class, pos)?.has_parent(type_name, ctx, pos)?;
        in_a_parent = in_a_parent || is_parent;
        if in_a_parent {
            break;
        }
    }

    if in_a_parent {
        Ok(())
    } else {
        let msg = if let Some(class) = in_class.last() {
            format!("Cannot access private {} of a {} while in a {}", name, type_name, class)
        } else {
            format!("Cannot access private {} of a {}", name, type_name)
        };
        Err(vec![TypeErr::new(pos, &msg)])
    }
}
