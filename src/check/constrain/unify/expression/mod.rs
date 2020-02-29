use crate::check::constrain::constraint::expected::Expect::{Collection, Expression, ExpressionAny};
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
use std::convert::TryFrom;

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
                let mut constr = substitute(&constraint.ids, &left, &right, constraints)?;
                unify_link(&mut constr, ctx, total)
            }
            _ => Err(vec![TypeErr::new(
                &ast.pos,
                &format!("Expected an expression but was {}", ast.node)
            )])
        },

        (Expression { ast }, Collection { ty }) => match &ast.node {
            Node::Set { elements } | Node::List { elements } | Node::Tuple { elements } => {
                for e in elements {
                    constraints.push("expression and collection", &Expected::try_from(e)?, ty);
                }
                unify_link(constraints, ctx, total)
            }
            _ => {
                let mut constr = substitute(&constraint.ids, &left, &right, constraints)?;
                unify_link(&mut constr, ctx, total)
            }
        },

        (Expression { ast: l_ast }, Expression { ast: r_ast }) =>
            match (&l_ast.node, &r_ast.node) {
                (Node::Set { elements: l_elements }, Node::Set { elements: r_elements })
                | (Node::List { elements: l_elements }, Node::List { elements: r_elements })
                | (Node::Tuple { elements: l_elements }, Node::Tuple { elements: r_elements }) =>
                    if l_elements.len() == r_elements.len() {
                        for (l, r) in l_elements.iter().zip(r_elements) {
                            constraints.push(
                                "collection expression",
                                &Expected::try_from(l)?,
                                &Expected::try_from(r)?
                            );
                        }
                        unify_link(constraints, ctx, total)
                    } else {
                        let msg = format!(
                            "Collection size differs, expected {}, was {}.",
                            l_elements.len(),
                            r_elements.len()
                        );
                        Err(vec![TypeErr::new(&left.pos, &msg)])
                    },
                _ => {
                    let mut constr = substitute(&constraint.ids, &left, &right, constraints)?;
                    unify_link(&mut constr, ctx, total)
                }
            },

        (Expression { .. }, _) => {
            let mut constraints = substitute(&constraint.ids, &left, &right, constraints)?;
            unify_link(&mut constraints, ctx, total)
        }

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
