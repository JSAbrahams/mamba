use crate::common::position::Position;
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::constraints::constraint::expected::Expect::{Collection, Expression,
                                                                     ExpressionAny, Truthy, Type};
use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::constraints::constraint::iterator::Constraints;

/// Substitute old expression with new
///
/// If old is a type, and new is an expression, we instead substitute new with
/// old. This is done to prevent substituting back in expressions after for
/// instance we conclude two sides of a constraint are trivially equal.
///
/// Also checks for expected nested within expected.
/// Does not, however, recursively traverse AST to check if they contain an AST.
pub fn substitute(
    old: &Expected,
    new: &Expected,
    constr: &Constraints,
    pos: &Position
) -> TypeResult<Constraints> {
    let mut substituted = Constraints::new(&[], &constr.in_class);
    let mut constr = constr.clone();
    let (old, new) = match (&old.expect, &new.expect) {
        (Type { type_name: lt }, Type { type_name: rt }) if lt != rt => {
            let msg = format!("Tried to substitute {} with {}", lt, rt);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }
        (Type { .. }, Type { .. })
        | (Truthy, Type { .. })
        | (Type { .. }, Truthy)
        | (Truthy, Truthy) => return Ok(constr),
        (Type { .. }, _)
        | (Truthy, _)
        | (Collection { .. }, _)
        | (ExpressionAny, Expression { .. }) => (new, old),
        _ => (old, new)
    };

    while let Some(mut constraint) = constr.pop_constr() {
        let (sub_l, parent) = recursive_substitute("l", &constraint.parent, old, new);
        let (sub_r, child) = recursive_substitute("r", &constraint.child, old, new);

        constraint.parent = parent;
        constraint.child = child;
        constraint.substituted = constraint.substituted || sub_l || sub_r;
        substituted.push_constr(&constraint)
    }

    Ok(substituted)
}

fn recursive_substitute(
    side: &str,
    expected: &Expected,
    old: &Expected,
    new: &Expected
) -> (bool, Expected) {
    macro_rules! replace {
        ($inner:expr, $new:expr) => {{
            let pos = format!("({}-{})", expected.pos.start, new.pos.start);
            let count = format!("[{}subst ({})]", if $inner { "inner " } else { "" }, side);
            println!("{:width$} {} {} <= {}", pos, count, expected.expect, $new.expect, width = 17);
        }};
    };

    if structurally_eq_not_type(&expected.expect, &old.expect) {
        replace!(false, new);
        return (true, new.clone());
    }

    match &expected.expect {
        Expect::Collection { ty } =>
            if structurally_eq_not_type(ty, &old.expect) {
                let new = Expected::new(&expected.pos, &Expect::Collection {
                    ty: Box::from(new.clone().expect)
                });
                replace!(true, new);
                (true, new)
            } else {
                (false, expected.clone())
            },
        Expect::Function { name, args } => {
            let mut any_substituted = false;
            let mut new_args = vec![];
            for arg in args {
                let (subs, arg) = recursive_substitute(side, arg, old, new);
                new_args.push(arg);
                any_substituted = any_substituted || subs;
            }
            (
                any_substituted,
                Expected::new(&expected.pos, &Expect::Function {
                    name: name.clone(),
                    args: new_args
                })
            )
        }
        Expect::Implements { type_name: name, args } => {
            let mut any_substituted = false;
            let mut new_args = vec![];
            for arg in args {
                let (subs, arg) = recursive_substitute(side, arg, old, new);
                new_args.push(arg);
                any_substituted = any_substituted || subs;
            }
            (
                any_substituted,
                Expected::new(&expected.pos, &Expect::Implements {
                    type_name: name.clone(),
                    args:      new_args
                })
            )
        }
        _ => (false, expected.clone())
    }
}

fn structurally_eq_not_type(inspected: &Expect, old: &Expect) -> bool {
    match inspected {
        Type { .. } => false,
        inspected => inspected.structurally_eq(&old) && inspected != &Truthy
    }
}
