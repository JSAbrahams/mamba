use crate::check::constrain::constraint::expected::Expect::{Expression, ExpressionAny};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::unify::link::reinsert;
use crate::check::constrain::unify::link::unify_link;
use crate::check::constrain::unify::unify_expression::substitute::substitute;
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
    total: usize
) -> Unified {
    match (&left.expect, &right.expect) {
        (Expression { ast }, ExpressionAny) => match &ast.node {
            Node::ConstructorCall { .. }
            | Node::FunctionCall { .. }
            | Node::PropertyCall { .. } => {
                // may be expression, defer in case substituted
                reinsert(constraints, constraint, total)?;
                unify_link(constraints, ctx, total)
            }
            node if node.trivially_expression() => {
                let mut constr = substitute(&constraint.idents, &left, &right, constraints)?;
                unify_link(&mut constr, ctx, total)
            }
            _ => Err(vec![TypeErr::new(
                &ast.pos,
                &format!("Expected an expression but was {}", ast.node)
            )])
        },

        (Expression { .. }, _) => {
            let mut constr = substitute(&constraint.idents, &left, &right, constraints)?;
            unify_link(&mut constr, ctx, total)
        }

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
