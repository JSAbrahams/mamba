use crate::common::position::Position;
use crate::type_checker::constraints::cons::Expect::*;
use crate::type_checker::constraints::cons::{Constraint, Constraints, Expect};
use crate::type_checker::type_result::{TypeErr, TypeResult};

/// Empties out constraints and puts them in a substituted list.
pub fn unify_link(constr: &Constraints, sub: &Constraints) -> TypeResult<Constraints> {
    let mut constraints = constr.clone();
    let mut sub = sub.clone();

    while let Some(constraint) = constraints.constraints.pop() {
        let unified = match (&constraint.0, &constraint.1) {
            (ExpressionAny, ExpressionAny) | (Truthy, Truthy) | (RaisesAny, RaisesAny) =>
                Ok(Constraints::from(&constraint)),

            (Expression { .. }, ExpressionAny) | (ExpressionAny, Expression { .. }) =>
                Ok(Constraints::from(&constraint)),
            (Expression { .. }, Expression { .. }) => unify_link(
                &substitute(&constraint.0, &constraint.1, &constraints)?,
                &unify_link(
                    &Constraints::from(&constraint),
                    &substitute(&constraint.0, &constraint.1, &sub)?.add_constraint(&constraint)
                )?
            ),

            (Type { type_name: left }, Type { type_name: right }) if left == right =>
                Ok(Constraints::from(&constraint)),
            (Type { type_name: left }, Type { type_name: right }) => Err(vec![TypeErr::new(
                &Position::default(),
                &format!("Types not equal: {}, {}", left, right)
            )]),

            (Type { .. }, Expression { .. }) | (Expression { .. }, Type { .. }) => unify_link(
                &substitute(&constraint.0, &constraint.1, constr)?,
                &substitute(&constraint.0, &constraint.1, &sub)?.add_constraint(&constraint)
            ),

            _ => panic!("Unexpected: {:?} == {:?}", constraint.0, constraint.1)
        }?;

        sub.append(&unified);
    }

    Ok(sub)
}

fn substitute(old: &Expect, new: &Expect, constraints: &Constraints) -> TypeResult<Constraints> {
    let mut constraints = constraints.clone();
    match &constraints.constraints.pop() {
        None => Ok(constraints.clone()),
        Some(Constraint(left, right)) => {
            let left = if left == old { new.clone() } else { left.clone() };
            let right = if right == old { new.clone() } else { right.clone() };
            let mut unified = Constraints::from(&Constraint(left, right));
            Ok(unified.append(&substitute(old, new, &constraints)?))
        }
    }
}
