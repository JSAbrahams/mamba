use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::Constraints;
use crate::type_checker::type_result::TypeResult;

pub fn substitute(old: &Expected, new: &Expected, constr: &Constraints) -> TypeResult<Constraints> {
    sub_inner(old, new, &mut constr.clone())
}

fn sub_inner(old: &Expected, new: &Expected, constr: &mut Constraints) -> TypeResult<Constraints> {
    let mut substituted = Constraints::new();

    while let Some(constraint) = constr.constraints.pop() {
        let (left, right) = (constraint.0, constraint.1);
        macro_rules! replace {
            () => {{
                let pos = format!("({}={})", old.pos, new.pos);
                println!(
                    "{:width$} [substitute] {} <= {}",
                    pos,
                    old.expect,
                    new.expect,
                    width = 32
                );
            }};
        };

        let left = if &left == old {
            replace!();
            Expected::new(&left.pos, &new.expect)
        } else {
            left
        };

        let right = if &right == old {
            replace!();
            Expected::new(&right.pos, &new.expect)
        } else {
            right
        };

        substituted.push(&left, &right);
    }

    Ok(substituted)
}
