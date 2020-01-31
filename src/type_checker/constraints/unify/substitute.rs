use crate::type_checker::constraints::constraint::expected::Expect::{Truthy, Type};
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::type_result::TypeResult;

/// Substitute old expression with new
///
/// If old is a type, and new is an expression, we instead substitute new with
/// old. This is done to prevent substituting back in expressions after for
/// instance we conclude two sides of a constraint are trivially equal.
pub fn substitute(old: &Expected, new: &Expected, constr: &Constraints) -> TypeResult<Constraints> {
    let total = constr.len();
    let mut substituted = Constraints::default();
    let mut constr = constr.clone();
    let (old, new) = match (&old.expect, &new.expect) {
        (Type { .. }, Type { .. }) => return Ok(constr),
        (Type { .. }, _) | (Truthy, _) => (new, old),
        _ => (old, new)
    };

    macro_rules! replace {
        ($side:expr, $constr:expr) => {{
            let pos = format!("({}={})", $constr.pos.start, new.pos.start);
            let count = format!("[substitute {} of {} ({})]", total - constr.len(), total, $side);
            println!("{:width$} {} {} <= {}", pos, count, $constr.expect, new.expect, width = 17);
        }};
    };

    while let Some(mut constraint) = constr.pop_constr() {
        if constraint.parent.expect.trivially_eq(&old.expect) {
            replace!("lhs", constraint.parent);
            constraint.replace_parent(&Expected::new(&constraint.parent.pos, &new.expect));
        }
        if constraint.child.expect.trivially_eq(&old.expect) {
            replace!("rhs", constraint.child);
            constraint.replace_child(&Expected::new(&constraint.child.pos, &new.expect));
        }

        substituted.push_constr(&constraint)
    }

    Ok(substituted)
}
