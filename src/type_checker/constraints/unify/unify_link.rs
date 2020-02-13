use crate::common::delimit::comma_delimited;
use crate::type_checker::constraints::constraint::expected::Expect::{Access, Collection,
                                                                     Expression, Function,
                                                                     Nullable, Stringy, Truthy,
                                                                     Type};
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::constraint::Constraint;
use crate::type_checker::constraints::unify::unify_direct::{is_direct, unify_direct};
use crate::type_checker::constraints::unify::unify_expression::unify_expression;
use crate::type_checker::constraints::unify::unify_function::unify_function;
use crate::type_checker::constraints::unify::unify_type::unify_type;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::Context;

/// Unifies all constraints.

/// We use a mutable reference to constraints for performance reasons.
/// Otherwise, we have to make a entirely new copy of the list of all
/// constraints each time we do a recursive call to unify link.
pub fn unify_link(constraints: &mut Constraints, ctx: &Context, total: usize) -> Unified {
    if let Some(constr) = &constraints.pop_constr() {
        let (left, right) = (constr.parent.clone(), constr.child.clone());
        let pos = format!("({}-{})", left.pos.start, right.pos.start);
        let unify = format!(
            "[unifying {}\\{}{}{}] ",
            total - constraints.len(),
            total,
            if constr.is_flag { " (fl)" } else { "" },
            if constr.is_sub { " (sub)" } else { "" }
        );
        let idens = if constr.idents.is_empty() {
            String::new()
        } else {
            format!(" [iden: {}] ", comma_delimited(&constr.idents))
        };
        println!("{:width$} {}{}{} = {}", pos, unify, idens, left.expect, right.expect, width = 15);

        match (&left.expect, &right.expect) {
            // trivially equal
            (left, right) if left == right => unify_link(constraints, ctx, total),

            // primitive and constructor substitutions
            // sometimes necessary when generating new constraints during unification
            (Expression { ast }, Expression { .. }) if is_direct(ast) =>
                unify_direct(&left, &right, constraints, ctx, total),
            (Expression { .. }, Expression { ast }) if is_direct(ast) =>
                unify_direct(&right, &left, constraints, ctx, total),

            (Function { .. }, Type { .. }) | (Access { .. }, _) =>
                unify_function(constr, &left, &right, constraints, ctx, total),
            (Type { .. }, Function { .. }) | (_, Access { .. }) =>
                unify_function(constr, &right, &left, constraints, ctx, total),

            (Expression { .. }, _) =>
                unify_expression(constr, &left, &right, constraints, ctx, total),
            (_, Expression { .. }) =>
                unify_expression(constr, &right, &left, constraints, ctx, total),

            (Type { .. }, _) | (_, Stringy) | (_, Truthy) | (_, Nullable) =>
                unify_type(&left, &right, constraints, ctx, total),
            (_, Type { .. }) | (Stringy, _) | (Truthy, _) | (Nullable, _) =>
                unify_type(&right, &left, constraints, ctx, total),
            (Collection { .. }, Collection { .. }) =>
                unify_type(&right, &left, constraints, ctx, total),

            _ => {
                let mut constr = reinsert(constraints, &constr, total)?;
                unify_link(&mut constr, ctx, total + 1)
            }
        }
    } else {
        Ok(constraints.clone())
    }
}

/// Reinsert constraint
///
/// The amount of attempts is a counter which states how often we allow
/// reinserts.
pub fn reinsert(constr: &mut Constraints, constraint: &Constraint, total: usize) -> Unified {
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
