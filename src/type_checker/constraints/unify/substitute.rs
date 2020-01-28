use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::type_result::TypeResult;

pub fn substitute(old: &Expected, new: &Expected, constr: &Constraints) -> TypeResult<Constraints> {
    let total = constr.len();
    let mut substituted = Constraints::default();
    let mut constr = constr.clone();
    macro_rules! replace {
        ($side:expr, $constr:expr) => {{
            let pos = format!("({}={})", $constr.pos.start, new.pos.start);
            let count = format!("[substitute {} of {} ({})]", total - constr.len(), total, $side);
            println!("{:width$} {} {} <= {}", pos, count, $constr.expect, new.expect, width = 15);
        }};
    };

    while let Some(mut constraint) = constr.pop_constr() {
        if &constraint.parent == old {
            replace!("lhs", constraint.parent);
            constraint.replace_parent(&Expected::new(&constraint.parent.pos, &new.expect));
        }
        if &constraint.child == old {
            replace!("rhs", constraint.child);
            constraint.replace_child(&Expected::new(&constraint.child.pos, &new.expect));
        }

        substituted.push_constr(&constraint)
    }

    Ok(substituted)
}
