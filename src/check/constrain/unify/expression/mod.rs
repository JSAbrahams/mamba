use EitherOrBoth::Both;
use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expect::{Expression, ExpressionAny, Tuple};
use crate::check::constrain::constraint::expected::{Expected, Expect};
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::expression::substitute::substitute;
use crate::check::constrain::unify::link::reinsert;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

mod substitute;

pub fn unify_expression(
    constraint: &Constraint,
    left: &Expected,
    right: &Expected,
    constraints: &mut Constraints,
    ctx: &Context,
    count: usize,
    total: usize,
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
                    let mut constr = substitute(&right, &left, constraints, count, total)?;
                    unify_link(&mut constr, ctx, total)
                }
                _ => Err(vec![TypeErr::new(&ast.pos, &format!("Expected an expression but was {}", ast.node))])
            },

        // Not sure if necessary, but exception made for tuple
        (Expression { ast: AST { node: Node::Tuple { elements: ast_elements }, .. } }, Tuple { elements }) |
        (Tuple { elements }, Expression { ast: AST { node: Node::Tuple { elements: ast_elements }, .. } }) => {
            for pair in ast_elements.iter().zip_longest(elements.iter()) {
                match &pair {
                    Both(ast, exp) => {
                        let expect = Expect::Expression { ast: ast.clone().clone() };
                        let l_ty = Expected::new(&left.pos, &expect);
                        constraints.push("tuple", &l_ty, &exp)
                    }
                    _ => {
                        let msg = format!("Expected tuple with {} elements, was {}", elements.len(), ast_elements.len());
                        return Err(vec![TypeErr::new(&left.pos, &msg)]);
                    }
                }
            }
            unify_link(constraints, ctx, total)
        }
        (Expression { ast: AST { node: Node::Tuple { elements: left_elements }, .. } },
            Expression { ast: AST { node: Node::Tuple { elements: right_elements }, .. } }) => {
            for pair in left_elements.iter().zip_longest(right_elements.iter()) {
                match &pair {
                    Both(left, right) => {
                        let r_expect = Expect::Expression { ast: right.clone().clone() };
                        let r_ty = Expected::new(&left.pos, &r_expect);
                        let r_expect = Expect::Expression { ast: left.clone().clone() };
                        let l_ty = Expected::new(&left.pos, &r_expect);
                        constraints.push("tuple", &r_ty, &l_ty)
                    }
                    _ => {
                        let msg = format!("Expected tuple with {} elements, was {}", left_elements.len(), right_elements.len());
                        return Err(vec![TypeErr::new(&left.pos, &msg)]);
                    }
                }
            }
            unify_link(constraints, ctx, total)
        }

        (Expression { .. }, _) => {
            let mut constraints = substitute(&right, &left, constraints, count, total)?;
            unify_link(&mut constraints, ctx, total)
        }
        _ => {
            let mut constraints = substitute(&left, &right, constraints, count, total)?;
            unify_link(&mut constraints, ctx, total)
        }
    }
}
