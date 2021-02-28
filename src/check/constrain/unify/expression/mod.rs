use std::convert::TryFrom;

use crate::check::constrain::constraint::expected::Expect::{Collection, Expression, ExpressionAny,
                                                            Tuple};
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
use itertools::{EitherOrBoth, Itertools};

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
        (Expression { ast }, ExpressionAny) | (ExpressionAny, Expression { ast }) =>
            match &ast.node {
                Node::FunctionCall { .. } | Node::PropertyCall { .. } => {
                    // may be expression, defer in case substituted
                    reinsert(constraints, constraint, total)?;
                    unify_link(constraints, ctx, total)
                }
                node if node.trivially_expression() => {
                    let mut constr = substitute(&constraint.ids, &left, &right, constraints, total)?;
                    unify_link(&mut constr, ctx, total)
                }
                _ => Err(vec![TypeErr::new(
                    &ast.pos,
                    &format!("Expected an expression but was {}", ast.node)
                )])
            },

        (Expression { ast }, Collection { ty }) | (Collection { ty }, Expression { ast }) => {
            let mut pushed = 0;
            match &ast.node {
                Node::Set { elements } | Node::List { elements } =>
                    for e in elements {
                        constraints.push("expression and collection", &Expected::try_from(e)?, ty);
                        pushed += 1;
                    },
                _ => {}
            }

            let mut constr = substitute(&constraint.ids, &left, &right, constraints, total)?;
            unify_link(&mut constr, ctx, total + pushed)
        }

        (Expression { ast }, Tuple { elements }) | (Tuple { elements }, Expression { ast }) => {
            let mut pushed = 0;
            match &ast.node {
                Node::Tuple { elements: tuple_els } => {
                    for pair in elements.iter().zip_longest(tuple_els.iter()) {
                        match pair {
                            EitherOrBoth::Both(tuple_exp, expr_ast) => {
                                constraints.push(
                                    "expression and tuple",
                                    tuple_exp,
                                    &Expected::try_from(expr_ast)?
                                );
                                pushed += 1;
                            }
                            _ => {
                                let msg = format!(
                                    "Expected tuple of size {}, was {}",
                                    elements.len(),
                                    tuple_els.len()
                                );
                                return Err(vec![TypeErr::new(&left.pos, &msg)]);
                            }
                        }
                    }
                }
                _ => {}
            }

            let mut constr = substitute(&constraint.ids, &left, &right, constraints, total)?;
            unify_link(&mut constr, ctx, total + pushed)
        }

        (Expression { ast: l_ast }, Expression { ast: r_ast }) => {
            let mut pushed = 0;
            match (&l_ast.node, &r_ast.node) {
                (Node::Set { elements: l_el }, Node::Set { elements: r_el })
                | (Node::List { elements: l_el }, Node::List { elements: r_el })
                | (Node::Tuple { elements: l_el }, Node::Tuple { elements: r_el }) =>
                    if l_el.len() == r_el.len() {
                        for (l, r) in l_el.iter().zip(r_el) {
                            let l_exp = Expected::try_from(l)?;
                            let r_exp = Expected::try_from(r)?;
                            constraints.push("collection expression", &l_exp, &r_exp);
                            pushed += 1;
                        }
                    } else {
                        let msg = format!(
                            "Expected collection with {} elements, was {}.",
                            l_el.len(),
                            r_el.len()
                        );
                        return Err(vec![TypeErr::new(&left.pos, &msg)]);
                    },
                _ => {}
            }

            let mut constr = substitute(&constraint.ids, &left, &right, constraints, total)?;
            unify_link(&mut constr, ctx, total + pushed)
        }

        (Expression { .. }, _) => {
            let mut constraints = substitute(&constraint.ids, &left, &right, constraints, total)?;
            unify_link(&mut constraints, ctx, total)
        }

        (l_exp, r_exp) => {
            let msg = format!("Expected '{}', found '{}'", l_exp, r_exp);
            Err(vec![TypeErr::new(&left.pos, &msg)])
        }
    }
}
