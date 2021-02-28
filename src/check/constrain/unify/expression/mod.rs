use crate::check::constrain::constraint::expected::Expect::{Expression, ExpressionAny};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::unify::expression::substitute::substitute;
use crate::check::constrain::unify::link::reinsert;
use crate::check::constrain::unify::link::unify_link;
use crate::check::constrain::Unified;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::Node;

mod substitute;

pub fn unify_expression(
    constraint: &Constraint,
    left: &Expected,
    right: &Expected,
    constraints: &mut Constraints,
    ctx: &Context,
    count: usize,
    total: usize
) -> Unified {
    match (&left.expect, &right.expect) {
        (Expression { ast }, ExpressionAny) | (ExpressionAny, Expression { ast }) =>
            match &ast.node {
                Node::FunctionCall { .. } | Node::PropertyCall { .. } => {
                    // may be expression, defer in case substituted
                    reinsert(constraints, constraint, total)?;
                    unify_link(constraints, ctx, total)
                }
                node if node.trivially_expression() => {
                    let mut constr = substitute(&left, &right, constraints, count, total)?;
                    unify_link(&mut constr, ctx, total)
                }
                _ => Err(vec![TypeErr::new(
                    &ast.pos,
                    &format!("Expected an expression but was {}", ast.node)
                )])
            },

        (Expression { .. }, _) => {
            let mut constraints = substitute(&left, &right, constraints, count, total)?;
            unify_link(&mut constraints, ctx, total)
        }

        (l_exp, r_exp) => {
            let msg = format!("Expected a '{}', was a '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
