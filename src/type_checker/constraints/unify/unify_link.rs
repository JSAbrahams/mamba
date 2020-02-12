use std::collections::HashSet;
use std::convert::TryFrom;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::constraint::Constraint;
use crate::type_checker::constraints::unify::substitute::substitute;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::function;
use crate::type_checker::context::function_arg::concrete::FunctionArg;
use crate::type_checker::context::ty;
use crate::type_checker::context::Context;
use crate::type_checker::ty_name::actual::ActualTypeName;
use crate::type_checker::ty_name::TypeName;
use crate::type_checker::util::comma_delimited;

/// Unifies all constraints.

/// We use a mutable reference to constraints for performance reasons.
/// Otherwise, we have to make a entirely new copy of the list of all
/// constraints each time we do a recursive call to unify link.
pub fn unify_link(constr: &mut Constraints, ctx: &Context, total: usize) -> Unified {
    if let Some(constraint) = &constr.pop_constr() {
        let (left, right) = (constraint.parent.clone(), constraint.child.clone());
        println!(
            "{:width$} [unifying {}\\{}{}{}]{} {} = {}",
            format!("({}-{})", left.pos.start, right.pos.start),
            total - constr.len(),
            total,
            if constraint.flagged { " (fl)" } else { "" },
            if constraint.substituted { " (sub)" } else { "" },
            if constraint.identifiers.is_empty() {
                String::new()
            } else {
                format!(" [iden: {}]", comma_delimited(&constraint.identifiers))
            },
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
                unify_link(constr, ctx, total + 1)
            }
            (Expression { ast: AST { node: Node::Real { .. }, .. } }, Expression { .. }) => {
                let type_name = TypeName::from(ty::concrete::FLOAT_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }
            (Expression { ast: AST { node: Node::Int { .. }, .. } }, Expression { .. }) => {
                let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }
            (Expression { ast: AST { node: Node::Str { .. }, .. } }, Expression { .. }) => {
                let type_name = TypeName::from(ty::concrete::STRING_PRIMITIVE);
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }
            (
                Expression { ast: AST { node: Node::ConstructorCall { name, .. }, .. } },
                Expression { .. }
            ) => {
                let type_name = TypeName::try_from(name)?;
                constr.eager_push(&right, &Expected::new(&left.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }

            (Expression { .. }, Expression { ast: AST { node: Node::Bool { .. }, .. } }) => {
                let type_name = TypeName::from(ty::concrete::BOOL_PRIMITIVE);
                constr.eager_push(&left, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }
            (Expression { .. }, Expression { ast: AST { node: Node::Real { .. }, .. } }) => {
                let type_name = TypeName::from(ty::concrete::FLOAT_PRIMITIVE);
                constr.eager_push(&left, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }
            (Expression { .. }, Expression { ast: AST { node: Node::Int { .. }, .. } }) => {
                let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
                constr.eager_push(&left, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }
            (Expression { .. }, Expression { ast: AST { node: Node::Str { .. }, .. } }) => {
                let type_name = TypeName::from(ty::concrete::STRING_PRIMITIVE);
                constr.eager_push(&left, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }
            (
                Expression { .. },
                Expression { ast: AST { node: Node::ConstructorCall { name, .. }, .. } }
            ) => {
                let type_name = TypeName::try_from(name)?;
                constr.eager_push(&left, &Expected::new(&right.pos, &Type { type_name }));
                unify_link(constr, ctx, total + 1)
            }

            // trivially equal
            (l_expect, r_expect) if l_expect.structurally_eq(r_expect) => {
                let mut constr =
                    substitute(&constraint.identifiers, &left, &right, &constr, &right.pos)?;
                unify_link(&mut constr, ctx, total)
            }

            (Expression { ast }, ExpressionAny) | (ExpressionAny, Expression { ast }) => match &ast
                .node
            {
                Node::ConstructorCall { .. }
                | Node::FunctionCall { .. }
                | Node::PropertyCall { .. } => {
                    // may be expression, defer in case substituted
                    reinsert(constr, constraint, total)?;
                    unify_link(constr, ctx, total)
                }
                node if node.trivially_expression() => {
                    let mut constr =
                        substitute(&constraint.identifiers, &left, &right, &constr, &right.pos)?;
                    unify_link(&mut constr, ctx, total)
                }
                _ => Err(vec![TypeErr::new(
                    &ast.pos,
                    &format!("Expected an expression but was {}", ast.node)
                )])
            },

            (Type { .. }, ExpressionAny) | (ExpressionAny, Type { .. }) =>
                unify_link(constr, ctx, total),
            (Truthy, ExpressionAny)
            | (ExpressionAny, Truthy)
            | (Stringy, ExpressionAny)
            | (ExpressionAny, Stringy) => unify_link(constr, ctx, total),

            (Expression { ast: l_ast }, Expression { ast: r_ast }) =>
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
                        unify_link(constr, ctx, total + l_e.len())
                    }
                    _ => {
                        let mut constr = substitute(
                            &constraint.identifiers,
                            &left,
                            &right,
                            &constr,
                            &right.pos
                        )?;
                        unify_link(&mut constr, ctx, total)
                    }
                },

            (Expression { .. }, Type { .. })
            | (Expression { .. }, Truthy)
            | (Expression { .. }, Stringy)
            | (Type { .. }, Expression { .. })
            | (Truthy, Expression { .. })
            | (Stringy, Expression { .. })
            | (Statement, Expression { .. })
            | (Expression { .. }, Statement) => {
                let mut constr =
                    substitute(&constraint.identifiers, &left, &right, &constr, &left.pos)?;
                unify_link(&mut constr, ctx, total)
            }

            (Expression { ast }, Collection { ty }) | (Collection { ty }, Expression { ast }) =>
                match &ast.node {
                    Node::Set { elements } | Node::Tuple { elements } | Node::List { elements } => {
                        for element in elements {
                            constr.eager_push(&Expected::from(element), &ty);
                        }
                        unify_link(constr, ctx, total + elements.len())
                    }
                    _ => {
                        let mut constr =
                            substitute(&constraint.identifiers, &left, &right, &constr, &left.pos)?;
                        unify_link(&mut constr, ctx, total)
                    }
                },

            (Type { type_name }, Truthy) | (Truthy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                expr_ty.function(&TypeName::from(function::concrete::TRUTHY), &left.pos)?;
                unify_link(constr, ctx, total)
            }
            (Type { type_name }, Stringy) | (Stringy, Type { type_name }) => {
                let expr_ty = ctx.lookup(type_name, &left.pos)?;
                expr_ty.function(&TypeName::from(function::concrete::STR), &left.pos)?;
                unify_link(constr, ctx, total)
            }
            (Type { type_name: l_ty }, Type { type_name: r_ty }) => {
                if l_ty.is_superset(r_ty)
                    || ctx.lookup(&r_ty, &right.pos)?.has_parent(&l_ty, ctx, &left.pos)?
                {
                    ctx.lookup(l_ty, &left.pos)?;
                    unify_link(constr, ctx, total)
                } else {
                    // TODO construct error based on type of constraint
                    let msg = format!("Expected '{}', found '{}'", l_ty, r_ty);
                    Err(vec![TypeErr::new(&left.pos, &msg)])
                }
            }

            (Type { type_name }, Raises { raises }) | (Raises { raises }, Type { type_name }) =>
                if raises.contains(type_name) {
                    unify_link(constr, ctx, total)
                } else {
                    let msg = format!(
                        "Unexpected raises '{}', must be one of: {}",
                        type_name,
                        comma_delimited(raises)
                    );
                    Err(vec![TypeErr::new(&left.pos, &msg)])
                },

            (Type { type_name }, Nullable) | (Nullable, Type { type_name }) =>
                if type_name.is_nullable() {
                    unify_link(constr, ctx, total)
                } else {
                    Err(vec![TypeErr::new(
                        &left.pos,
                        &format!("Expected '{}', found '{}'", type_name.as_nullable(), type_name)
                    )])
                },

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

                unify_link(constr, ctx, total + count)
            }

            (Type { type_name }, Collection { ty }) | (Collection { ty }, Type { type_name }) => {
                let (mut constr, added) = check_iter(type_name, ty, ctx, constr, &left.pos)?;
                unify_link(&mut constr, ctx, total + added)
            }
            (Collection { ty: l_ty }, Collection { ty: r_ty }) => {
                constr.eager_push(&l_ty, &r_ty);
                unify_link(constr, ctx, total + 1)
            }

            (_, Access { entity, name }) =>
                unify_access(constraint, &left, entity, name, &left.pos, constr, ctx, total),
            (Access { entity, name }, _) =>
                unify_access(constraint, &right, entity, name, &right.pos, constr, ctx, total),

            (Nullable, Nullable) => unify_link(constr, ctx, total),
            (Nullable, Expression { ast: AST { node: Node::Undefined, .. } }) =>
                unify_link(constr, ctx, total),
            (Expression { ast: AST { node: Node::Undefined, .. } }, Nullable) =>
                unify_link(constr, ctx, total),

            (Truthy, Stringy) | (Stringy, Truthy) => unify_link(constr, ctx, total),
            (Stringy, Nullable) | (Nullable, Stringy) => unify_link(constr, ctx, total),

            _ => {
                let mut constr = reinsert(constr, &constraint, total)?;
                unify_link(&mut constr, ctx, total + 1)
            }
        }
    } else {
        Ok(constr.clone())
    }
}

fn unify_access(
    constraint: &Constraint,
    expected: &Expected,
    entity: &Expected,
    name: &Expected,
    pos: &Position,
    constr: &mut Constraints,
    ctx: &Context,
    total: usize
) -> Unified {
    if let Type { type_name: entity_name } = &entity.expect {
        match &name.expect {
            Field { name } => {
                let field = ctx.lookup(entity_name, pos)?.field(name, pos)?;
                if field.private {
                    let name = ActualTypeName::new(&field.name, &[]);
                    check_if_parent(&name, &constr.in_class, entity_name, ctx, pos)?;
                }
                let field_ty_exp = if let Some(ty) = field.ty {
                    Expected::new(pos, &Type { type_name: ty.clone() })
                } else {
                    Expected::new(pos, &Statement)
                };
                constr.eager_push(&expected, &field_ty_exp);
                unify_link(constr, ctx, total)
            }
            Function { name, args } => {
                let expr_ty = ctx.lookup(entity_name, pos)?;
                let possible_fun = expr_ty.function(&name, pos)?;

                for function in &possible_fun {
                    if function.private {
                        check_if_parent(&function.name, &constr.in_class, &entity_name, ctx, pos)?;
                    }

                    constr.eager_push(
                        &expected,
                        &Expected::new(
                            pos,
                            &if let Some(ty) = function.ty() {
                                Type { type_name: ty }
                            } else {
                                Statement
                            }
                        )
                    );
                }

                let possible_args: HashSet<Vec<FunctionArg>> =
                    possible_fun.iter().map(|f| f.arguments.clone()).collect();
                let (mut constr, added) = unify_fun_arg(&possible_args, &args, &constr, pos)?;
                unify_link(&mut constr, ctx, total + added)
            }
            _ => {
                let mut constr = reinsert(constr, &constraint, total)?;
                unify_link(&mut constr, ctx, total + 1)
            }
        }
    } else {
        let mut constr = reinsert(constr, &constraint, total)?;
        unify_link(&mut constr, ctx, total + 1)
    }
}

/// Reinsert constraint
///
/// The amount of attempts is a counter which states how often we allow
/// reinserts.
fn reinsert(constr: &mut Constraints, constraint: &Constraint, total: usize) -> Unified {
    let pos = format!("({}-{})", constraint.parent.pos.start, constraint.child.pos.start);
    let count = format!("[reinserting {}\\{}]", total - constr.len(), total);
    println!(
        "{:width$} {} {} = {}",
        pos,
        count,
        constraint.parent.expect,
        constraint.child.expect,
        width = 17
    );

    // Defer to later point
    constr.reinsert(&constraint)?;
    Ok(constr.clone())
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
    ty: &Expected,
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
        constr.eager_push(&ty, &Expected::new(&pos, &Type { type_name: f_ret_ty }));
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
