use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::type_result::TypeResult;

pub fn substitute(old: &Expected, new: &Expected, constr: &Constraints) -> TypeResult<Constraints> {
    sub_inner(old, new, &mut constr.clone())
}

fn sub_inner(old: &Expected, new: &Expected, constr: &mut Constraints) -> TypeResult<Constraints> {
    let total = constr.len();
    macro_rules! replace {
        () => {{
            let pos = format!("({}={})", old.pos.start, new.pos.start);
            let count = format!("[substitute {} of {}]", total - constr.len(), total);
            println!("{:width$} {} {} <= {}", pos, count, old.expect, new.expect, width = 17);
        }};
    };

    let mut substituted = Constraints::default();
    while let Some(mut constraint) = constr.pop_constr() {
        if &constraint.parent == old {
            replace!();
            constraint.replace_left(&Expected::new(&constraint.parent.pos, &new.expect));
        }
        if &constraint.child == old {
            replace!();
            constraint.replace_left(&Expected::new(&constraint.child.pos, &new.expect));
        }

        substituted.push_constr(&constraint)
    }

    Ok(substituted)
}
