use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::type_result::TypeResult;

pub fn substitute(old: &Expected, new: &Expected, constr: &Constraints) -> TypeResult<Constraints> {
    sub_inner(old, new, &mut constr.clone())
}

fn sub_inner(old: &Expected, new: &Expected, constr: &mut Constraints) -> TypeResult<Constraints> {
    let mut substituted = Constraints::default();
    let total = constr.len();

    while let Some(mut constraint) = constr.pop_constr() {
        macro_rules! replace {
            () => {{
                println!(
                    "{:width$} [substitute {} of {}] {} <= {}",
                    format!("({}={})", old.pos, new.pos),
                    total - constr.len(),
                    total,
                    old.expect,
                    new.expect,
                    width = 32
                );
            }};
        };

        if &constraint.left == old {
            replace!();
            constraint.replace_left(&Expected::new(&constraint.left.pos, &new.expect));
        }

        if &constraint.right == old {
            replace!();
            constraint.replace_left(&Expected::new(&constraint.right.pos, &new.expect));
        }

        substituted.push_constr(&constraint)
    }

    Ok(substituted)
}
