use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::constraints::constraint::expected::Expect::{Collection, Expression,
                                                                     ExpressionAny, Nullable,
                                                                     Statement, Stringy, Truthy,
                                                                     Type};
use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::constraints::constraint::iterator::Constraints;

/// Substitute old expression with new
///
/// Identifiers signals when we should stop substituting variables.
///
/// If old is a type, and new is an expression, we instead substitute new with
/// old. This is done to prevent substituting back in expressions after for
/// instance we conclude two sides of a constraint are trivially equal.
///
/// Also checks for expected nested within expected.
/// Does not, however, recursively traverse AST to check if they contain an AST.
pub fn substitute(
    identifiers: &[String],
    old: &Expected,
    new: &Expected,
    constr: &Constraints,
    pos: &Position
) -> TypeResult<Constraints> {
    let mut constr = constr.clone();
    let (old, new) = match (&old.expect, &new.expect) {
        (Type { type_name: lt }, Type { type_name: rt }) if lt != rt => {
            let msg = format!("Tried to substitute {} with {}", lt, rt);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }
        (Truthy, _)
        | (Collection { .. }, _)
        | (ExpressionAny, Expression { .. })
        | (_, Expression { ast: AST { node: Node::Id { .. }, .. } })
        | (Stringy, _)
        | (Nullable, _)
        | (Statement, _) => (new, old),
        _ => (old, new)
    };

    let (old, new) = if let Type { .. } = &old.expect { (new, old) } else { (old, new) };

    // TODO deal with tuples of identifiers
    let mut encountered = false;
    let mut substituted = Constraints::new(&[], &constr.in_class);
    while let Some(mut constraint) = constr.pop_constr() {
        encountered = encountered
            || !constraint.identifiers.is_empty()
                && constraint.identifiers == Vec::from(identifiers);
        if !encountered {
            let (sub_l, parent) = recursive_substitute("l", &constraint.parent, old, new);
            let (sub_r, child) = recursive_substitute("r", &constraint.child, old, new);

            constraint.parent = parent;
            constraint.child = child;
            constraint.substituted = constraint.substituted || sub_l || sub_r;
        }

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
        Expect::Access { entity, name } => {
            let (subs_e, entity) = recursive_substitute(side, entity, old, new);
            let (sub_n, name) = recursive_substitute(side, name, old, new);
            let expect = Expect::Access { entity: Box::from(entity), name: Box::from(name) };
            (subs_e || sub_n, Expected::new(&expected.pos, &expect))
        }
        Expect::Collection { ty } => {
            let (subs_ty, ty) = recursive_substitute(side, ty, old, new);
            let expect = Expect::Collection { ty: Box::from(ty.clone()) };
            (subs_ty, Expected::new(&expected.pos, &expect))
        }
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
        _ => (false, expected.clone())
    }
}

fn structurally_eq_not_type(inspected: &Expect, old: &Expect) -> bool {
    match inspected {
        Type { .. } => false,
        inspected => inspected.structurally_eq(&old) && inspected != &Truthy
    }
}
