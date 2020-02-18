use crate::check::context::clss::HasParent;
use crate::check::context::name::{DirectName, NameUnion};
use crate::check::context::{Context, LookupClass};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

pub fn check_is_parent(
    field: &NameUnion,
    in_class: &Vec<DirectName>,
    object_class: &NameUnion,
    ctx: &Context,
    pos: &Position
) -> TypeResult<()> {
    let mut in_a_parent = false;
    for class in in_class {
        let is_parent = ctx.class(class, pos)?.has_parent(object_class, ctx, pos)?;
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
