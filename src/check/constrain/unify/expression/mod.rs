use EitherOrBoth::Both;
use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expect::{Expression, Tuple};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::expression::substitute::substitute;
use crate::check::constrain::unify::finished::Finished;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub mod substitute;

pub fn unify_expression(constraint: &Constraint, constraints: &mut Constraints, finished: &mut Finished, ctx: &Context, count: usize, total: usize) -> Unified {
    let (left, right) = (&constraint.parent, &constraint.child);
    match (&left.expect, &right.expect) {
        // Not sure if necessary, but exception made for tuple
        (Tuple { elements }, Expression { ast: AST { node: Node::Tuple { elements: ast_elements }, .. } }) |
        (Expression { ast: AST { node: Node::Tuple { elements: ast_elements }, .. } }, Tuple { elements }) => {
            substitute(constraints, left, right, count, total)?;

            for pair in ast_elements.iter().cloned().zip_longest(elements.iter()) {
                match &pair {
                    Both(ast, exp) => {
                        let expect = Expression { ast: ast.clone() };
                        constraints.push("tuple", &Expected::new(left.pos, &expect), exp)
                    }
                    _ => {
                        let msg = format!("Expected tuple with {} elements, was {}", elements.len(), ast_elements.len());
                        return Err(vec![TypeErr::new(left.pos, &msg)]);
                    }
                }
            }
        }

        (Expression { .. }, _) => substitute(constraints, right, left, count, total)?,
        _ => substitute(constraints, left, right, count, total)?
    }

    unify_link(constraints, finished, ctx, total)
}
