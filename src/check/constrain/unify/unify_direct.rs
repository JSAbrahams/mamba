use crate::check::constrain::constraint::expected::Expect::{Expression, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::unify::unify_link::unify_link;
use crate::check::constrain::Unified;
use crate::check::context::{clss, Context};
use crate::check::result::TypeErr;
use crate::check::ty;
use crate::parse::ast::{Node, AST};
use std::convert::TryFrom;

pub fn is_direct(node: &AST) -> bool {
    match &node.node {
        Node::Str { .. }
        | Node::Int { .. }
        | Node::Bool { .. }
        | Node::Real { .. }
        | Node::ConstructorCall { .. } => true,
        _ => false
    }
}

/// Unify expression directly.
///
/// Most of these constraints are generated during the generation stage.
/// However, for newly generated constraints during the unification stage,
/// this is necessary.
pub fn unify_direct(
    direct: &Expected,
    expt: &Expected,
    constraints: &mut Constraints,
    ctx: &Context,
    total: usize
) -> Unified {
    let node = if let Expression { ast } = &direct.expect {
        ast.node.clone()
    } else {
        let msg = format!("Expected expression, found '{}'", &direct.expect);
        return Err(vec![TypeErr::new(&direct.pos, &msg)]);
    };

    match (&node, &expt.expect) {
        (Node::Bool { .. }, Expression { .. }) => {
            let ty = ty::Type::from(clss::BOOL_PRIMITIVE);
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { ty }));
            unify_link(constraints, ctx, total + 1)
        }
        (Node::Real { .. }, Expression { .. }) => {
            let ty = ty::Type::from(clss::FLOAT_PRIMITIVE);
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { ty }));
            unify_link(constraints, ctx, total + 1)
        }
        (Node::Int { .. }, Expression { .. }) => {
            let ty = ty::Type::from(clss::INT_PRIMITIVE);
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { ty }));
            unify_link(constraints, ctx, total + 1)
        }
        (Node::Str { .. }, Expression { .. }) => {
            let ty = ty::Type::from(clss::STRING_PRIMITIVE);
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { ty }));
            unify_link(constraints, ctx, total + 1)
        }
        (Node::ConstructorCall { name, .. }, Expression { .. }) => {
            let ty = ty::Type::try_from(name)?;
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { ty }));
            unify_link(constraints, ctx, total + 1)
        }

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&direct.pos, &msg)])
        }
    }
}
