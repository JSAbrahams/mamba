use crate::parser::ast::Node;
use crate::type_checker::checker_result::TypeErr;
use crate::type_checker::constraints::constraint::expected::Expect::{Collection, Expression,
                                                                     ExpressionAny};
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::constraint::Constraint;
use crate::type_checker::constraints::unify::unify_expression::substitute::substitute;
use crate::type_checker::constraints::unify::unify_link::reinsert;
use crate::type_checker::constraints::unify::unify_link::unify_link;
use crate::type_checker::constraints::Unified;
use crate::type_checker::context::Context;
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

        (Expression { ast: l_ast }, Expression { ast: r_ast }) =>
            match (&l_ast.node, &r_ast.node) {
                (Node::Set { elements: l_e }, Node::Set { elements: r_e })
                | (Node::List { elements: l_e }, Node::List { elements: r_e })
                | (Node::Tuple { elements: l_e }, Node::Tuple { elements: r_e }) => {
                    for pair in l_e.iter().zip_longest(r_e.iter()) {
                        match pair {
                            EitherOrBoth::Both(l, r) =>
                                constraints.eager_push(&Expected::from(l), &Expected::from(r)),
                            EitherOrBoth::Left(e) | EitherOrBoth::Right(e) =>
                                return Err(vec![TypeErr::new(&e.pos, "Unexpected element")]),
                        }
                    }
                    unify_link(constraints, ctx, total + l_e.len())
                }
                _ => {
                    let mut constr = substitute(&constraint.idents, &left, &right, constraints)?;
                    unify_link(&mut constr, ctx, total)
                }
            },

        (Expression { ast }, Collection { ty }) => match &ast.node {
            Node::Set { elements } | Node::Tuple { elements } | Node::List { elements } => {
                for element in elements {
                    constraints.eager_push(&Expected::from(element), &ty);
                }
                unify_link(constraints, ctx, total + elements.len())
            }
            _ => {
                let mut constr = substitute(&constraint.idents, &left, &right, constraints)?;
                unify_link(&mut constr, ctx, total)
            }
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
