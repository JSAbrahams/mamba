use EitherOrBoth::Both;
use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::{Constraint, ConstrVariant};
use crate::check::constrain::constraint::expected::Expect::{Expression, Tuple};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::iterator::Constraints;
use crate::check::constrain::Unified;
use crate::check::constrain::unify::expression::substitute::substitute;
use crate::check::constrain::unify::link::unify_link;
use crate::check::context::Context;
use crate::check::result::TypeErr;
use crate::parse::ast::{AST, Node};

pub mod substitute;

pub fn unify_expression(constraint: &Constraint, constraints: &mut Constraints, ctx: &Context, count: usize, total: usize) -> Unified {
    let (left, right) = (&constraint.left, &constraint.right);
    match (&left.expect, &right.expect) {
        // Not sure if necessary, but exception made for tuple
        (Tuple { elements }, Expression { ast: AST { node: Node::Tuple { elements: ast_elements }, .. } }) |
        (Expression { ast: AST { node: Node::Tuple { elements: ast_elements }, .. } }, Tuple { elements }) => {
            let mut constraints = substitute(left, right, constraints, count, total)?;

            for pair in ast_elements.iter().cloned().zip_longest(elements.iter()) {
                match &pair {
                    Both(ast, exp) => {
                        let expect = Expression { ast: ast.clone() };
                        let l_ty = Expected::new(left.pos, &expect);
                        constraints.push("tuple", &l_ty, exp)
                    }
                    _ => {
                        let msg = format!("Expected tuple with {} elements, was {}", elements.len(), ast_elements.len());
                        return Err(vec![TypeErr::new(left.pos, &msg)]);
                    }
                }
            }

            unify_link(&mut constraints, ctx, total)
        }

        (Expression { .. }, _) if constraint.superset == ConstrVariant::Left || constraint.superset == ConstrVariant::Either => {
            let mut constraints = substitute(right, left, constraints, count, total)?;
            unify_link(&mut constraints, ctx, total)
        }
        _ => {
            let mut constraints = substitute(left, right, constraints, count, total)?;
            unify_link(&mut constraints, ctx, total)
        }
    }
}
