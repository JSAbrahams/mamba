use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::type_result::TypeResult;

pub fn substitute(old: &Expected, new: &Expected, constr: &Constraints) -> TypeResult<Constraints> {
    let total = constr.len();
    let mut substituted = Constraints::default();
    let mut constr = constr.clone();

    while let Some(mut constraint) = constr.pop_constr() {
        macro_rules! replace {
            ($side:expr) => {{
                let pos = format!("({}={})", constraint.parent.pos.start, new.pos.start);
                let count =
                    format!("[substitute {} of {} ({})]", total - constr.len(), total, $side);
                println!(
                    "{:width$} {} {} <= {}",
                    pos,
                    count,
                    constraint.parent.expect,
                    new.expect,
                    width = 15
                );
            }};
        };

        if &constraint.parent == old {
            replace!("lhs");
            constraint.replace_parent(&Expected::new(&constraint.parent.pos, &new.expect));
        }
        if &constraint.child == old {
            replace!("rhs");
            constraint.replace_child(&Expected::new(&constraint.child.pos, &new.expect));
        }

        substituted.push_constr(&constraint)
    }

    Ok(substituted)
}
