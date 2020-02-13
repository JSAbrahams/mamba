use crate::parser::ast::{Node, AST};
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::expected::Expect::{Expression, Type};
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::unify::unify_link::unify_link;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::{ty, Context};
use crate::type_checker::ty_name::TypeName;
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
            let type_name = TypeName::from(ty::concrete::BOOL_PRIMITIVE);
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { type_name }));
            unify_link(constraints, ctx, total + 1)
        }
        (Node::Real { .. }, Expression { .. }) => {
            let type_name = TypeName::from(ty::concrete::FLOAT_PRIMITIVE);
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { type_name }));
            unify_link(constraints, ctx, total + 1)
        }
        (Node::Int { .. }, Expression { .. }) => {
            let type_name = TypeName::from(ty::concrete::INT_PRIMITIVE);
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { type_name }));
            unify_link(constraints, ctx, total + 1)
        }
        (Node::Str { .. }, Expression { .. }) => {
            let type_name = TypeName::from(ty::concrete::STRING_PRIMITIVE);
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { type_name }));
            unify_link(constraints, ctx, total + 1)
        }
        (Node::ConstructorCall { name, .. }, Expression { .. }) => {
            let type_name = TypeName::try_from(name)?;
            constraints.eager_push(&expt, &Expected::new(&direct.pos, &Type { type_name }));
            unify_link(constraints, ctx, total + 1)
        }

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&direct.pos, &msg)])
        }
    }
}
