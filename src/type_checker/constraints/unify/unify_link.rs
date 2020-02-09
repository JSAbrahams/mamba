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
use crate::type_checker::context::ty;
use crate::type_checker::context::Context;
use crate::type_checker::ty_name::actual::ActualTypeName;
use crate::type_checker::ty_name::TypeName;
use crate::type_checker::util::comma_delimited;
use std::convert::TryFrom;

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
            // primitive and constructor substitutions
            // sometimes necessary when generating new constraints during unification
            (Expression { ast: AST { node: Node::Bool { .. }, .. } }, Expression { .. }) => {
                let type_name = TypeName::from(ty::concrete::BOOL_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }
            (Expression { ast: AST { node: Node::Real { .. }, .. } }, Expression { .. }) => {
                let type_name = TypeName::from(ty::concrete::FLOAT_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }
            (Expression { ast: AST { node: Node::Int { .. }, .. } }, Expression { .. }) => {
                let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }
            (Expression { ast: AST { node: Node::Str { .. }, .. } }, Expression { .. }) => {
                let type_name = TypeName::from(ty::concrete::STRING_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }
            (
                Expression { ast: AST { node: Node::ConstructorCall { name, .. }, .. } },
                Expression { .. }
            ) => {
                let type_name = TypeName::try_from(name)?;
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }

            (Expression { .. }, Expression { ast: AST { node: Node::Bool { .. }, .. } }) => {
                let type_name = TypeName::from(ty::concrete::BOOL_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }
            (Expression { .. }, Expression { ast: AST { node: Node::Real { .. }, .. } }) => {
                let type_name = TypeName::from(ty::concrete::FLOAT_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }
            (Expression { .. }, Expression { ast: AST { node: Node::Int { .. }, .. } }) => {
                let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }
            (Expression { .. }, Expression { ast: AST { node: Node::Str { .. }, .. } }) => {
                let type_name = TypeName::from(ty::concrete::STRING_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }
            (
                Expression { .. },
                Expression { ast: AST { node: Node::ConstructorCall { name, .. }, .. } }
            ) => {
                let type_name = TypeName::try_from(name)?;
                constr.eager_push(&right, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, sub, ctx, total + 1)
            }

            // trivially equal
            (l_expect, r_expect) if l_expect.structurally_eq(r_expect) => {
                let mut sub = substitute(&left, &right, &sub, &left.pos)?;
                sub.push_constr(constraint);

                let mut constr = substitute(&left, &right, &constr, &right.pos)?;
                unify_link(&mut constr, &mut sub, ctx, total)
            }

            (Expression { ast }, ExpressionAny) | (ExpressionAny, Expression { ast }) =>
                if ast.node.is_expression() {
                    // TODO if function call check if return type expression
                    let mut sub = substitute(&left, &right, &sub, &left.pos)?;
                    sub.push_constr(constraint);

                    let mut constr = substitute(&left, &right, &constr, &right.pos)?;
                    unify_link(&mut constr, &mut sub, ctx, total)
                } else {
                    Err(vec![TypeErr::new(
                        &ast.pos,
                        &format!("Expected expression but was {}", ast.node.name())
                    )])
                },

            (Type { .. }, ExpressionAny) | (ExpressionAny, Type { .. }) =>
                unify_link(constr, sub, ctx, total),
            (Truthy, ExpressionAny)
            | (ExpressionAny, Truthy)
            | (Stringy, ExpressionAny)
            | (ExpressionAny, Stringy) => unify_link(constr, sub, ctx, total),

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

            (Expression { ast: l_ast }, Expression { ast: r_ast }) => {
                match (&l_ast.node, &r_ast.node) {
                    (Node::Set { elements: l_e }, Node::Set { elements: r_e })
                    | (Node::List { elements: l_e }, Node::List { elements: r_e })
                    | (Node::Tuple { elements: l_e }, Node::Tuple { elements: r_e }) => {
                        for pair in l_e.iter().zip_longest(r_e.iter()) {
                            match pair {
                                EitherOrBoth::Both(l, r) =>
                                    constr.eager_push(&Expected::from(l), &Expected::from(r)),
                                EitherOrBoth::Left(e) | EitherOrBoth::Right(e) =>
                                    return Err(vec![TypeErr::new(&e.pos, "Unexpected element")]),
                            }
                        }
                        unify_link(constr, &sub, ctx, total + l_e.len())
                    }
                    _ => {
                        let mut sub = substitute(&left, &right, &sub, &left.pos)?;
                        sub.push_constr(constraint);

                        let mut constr = substitute(&left, &right, &constr, &right.pos)?;
                        unify_link(&mut constr, &sub, ctx, total)
                    }
                }
            }

            (Expression { .. }, Type { .. })
            | (Expression { .. }, Truthy)
            | (Expression { .. }, Stringy) => {
                let mut sub = substitute(&left, &right, &sub, &left.pos)?;
                sub.push_constr(constraint);

                let mut constr = substitute(&left, &right, &constr, &right.pos)?;
                unify_link(&mut constr, &sub, ctx, total)
            }

            (Type { .. }, Expression { ast })
            | (Truthy, Expression { ast })
            | (Stringy, Expression { ast }) =>
                if ast.node.is_expression() {
                    let mut sub = substitute(&left, &right, &sub, &left.pos)?;
                    sub.push_constr(constraint);

                    let mut constr = substitute(&left, &right, &constr, &left.pos)?;
                    unify_link(&mut constr, &sub, ctx, total)
                } else {
                    Err(vec![TypeErr::new(
                        &ast.pos,
                        &format!("Expected expression but was {}", ast.node.name())
                    )])
                },

            (Expression { ast }, Collection { ty }) | (Collection { ty }, Expression { ast }) =>
                match &ast.node {
                    Node::Set { elements } | Node::Tuple { elements } | Node::List { elements } => {
                        for element in elements {
                            constr.eager_push(
                                &Expected::from(element),
                                &Expected::new(&element.pos, &ty)
                            );
                        }
                        unify_link(constr, &sub, ctx, total + elements.len())
                    }
                    _ =>
                        if ast.node.is_expression() {
                            let mut sub = substitute(&left, &right, &sub, &left.pos)?;
                            sub.push_constr(constraint);
                            let mut constr = substitute(&left, &right, &constr, &left.pos)?;
                            unify_link(&mut constr, &sub, ctx, total)
                        } else {
                            Err(vec![TypeErr::new(
                                &ast.pos,
                                &format!("Expected expression but was {}", ast.node.name())
                            )])
                        },
                },

            (Type { type_name }, Truthy) | (Truthy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                expr_ty.function(&TypeName::from(function::concrete::TRUTHY), &left.pos)?;
                unify_link(constr, sub, ctx, total)
            }
            (Type { type_name }, Stringy) | (Stringy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                expr_ty.function(&TypeName::from(function::concrete::STR), &left.pos)?;
                unify_link(constr, sub, ctx, total)
            }
            (Type { type_name: l_ty }, Type { type_name: r_ty }) => {
                if l_ty.is_superset(r_ty)
                    || ctx.lookup(&r_ty, &right.pos)?.has_parent(&l_ty, ctx, &left.pos)?
                {
                    ctx.lookup(l_ty, &left.pos)?;
                    unify_link(constr, sub, ctx, total)
                } else {
                    // TODO construct error based on type of constraint
                    let msg = format!("Expected a {} but was a {}", l_ty, r_ty);
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

            (Type { type_name }, Raises { raises }) | (Raises { raises }, Type { type_name }) =>
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
                let (mut constr, added) = check_iter(type_name, ty, ctx, constr, &left.pos)?;
                unify_link(&mut constr, sub, ctx, total + added)
            }
            (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
                constr
                    .eager_push(&Expected::new(&left.pos, l_ty), &Expected::new(&right.pos, r_ty));
                unify_link(constr, sub, ctx, total + 1)
            }

            (Truthy, Stringy) | (Stringy, Truthy) => unify_link(constr, sub, ctx, total),

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
                    constr.eager_push(&Expected::new(&expected.pos, &Type { type_name }), &expected)
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
) -> TypeResult<(Constraints, usize)> {
    let f_name = TypeName::from(function::concrete::ITER);
    let mut added = 0;

    for fun in ctx.lookup(type_name, pos)?.function(&f_name, pos)? {
        let msg = format!("{} __iter__ type undefined", type_name);
        let f_ret_ty = fun.ty().ok_or_else(|| TypeErr::new(&pos, &msg))?;

        let f_name = TypeName::from(function::concrete::NEXT);
        for fun in ctx.lookup(&f_ret_ty, pos)?.function(&f_name, pos)? {
            let f_ret_ty = fun.ty().ok_or_else(|| TypeErr::new(&pos, &msg))?;
            added += 1;
            constr.eager_push(
                &Expected::new(&pos, &Type { type_name: type_name.clone() }),
                &Expected::new(&pos, &Type { type_name: f_ret_ty.clone() })
            );
        }
        added += 1;
        constr.eager_push(
            &Expected::new(&pos, &ty),
            &Expected::new(&pos, &Type { type_name: f_ret_ty })
        );
    }

    Ok((constr.clone(), added))
}

fn check_if_parent(
    field: &ActualTypeName,
    in_class: &Vec<TypeName>,
    object_class: &TypeName,
    ctx: &Context,
    pos: &Position
) -> TypeResult<()> {
    let mut in_a_parent = false;
    for class in in_class {
        let is_parent = ctx.lookup(&class, pos)?.has_parent(object_class, ctx, pos)?;
        in_a_parent = in_a_parent || is_parent;
        if in_a_parent {
            break;
        }
    }

    if in_a_parent {
        Ok(())
    } else {
        let msg = if let Some(class) = in_class.last() {
            format!("Cannot access private {} of a {} while in a {}", field, object_class, class)
        } else {
            format!("Cannot access private {} of a {}", field, object_class)
        };
        Err(vec![TypeErr::new(pos, &msg)])
    }
}
