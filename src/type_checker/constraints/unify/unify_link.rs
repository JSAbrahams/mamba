use crate::common::position::Position;
use crate::parser::ast::AST;
use crate::type_checker::constraints::cons::Expect::*;
use crate::type_checker::constraints::cons::{Constraint, Constraints};
use crate::type_checker::type_result::{TypeErr, TypeResult};

/// Empties out constraints and puts them in a substituted list.
pub fn unify_link(constr: &Constraints, sub: &Constraints) -> TypeResult<Constraints> {
    let mut constraints = constr.clone();
    let constraint = if let Some(constr) = constraints.constraints.pop() {
        constr.clone()
    } else {
        return Ok(constraints.clone());
    };

    let sub = sub.add(&constraint.0, &constraint.1);
    match (&constraint.0.clone(), &constraint.1.clone()) {
        (ExpressionAny, ExpressionAny) => Ok(sub),
        (Truthy, Truthy) => Ok(sub),
        (RaisesAny, RaisesAny) => Ok(sub),

        (Expression { ast: left }, Expression { ast: right }) => unify_link(
            &substitute(left, right, &constraints)?,
            &unify_link(&Constraints::from(constraint), &substitute(left, right, &sub)?)?
        ),

        (Type { type_name: left }, Type { type_name: right }) if left == right => Ok(sub),
        (Type { type_name: left }, Type { type_name: right }) => Err(vec![TypeErr::new(
            &Position::default(),
            &format!("Types not equal: {}, {}", left, right)
        )]),

        _ => Ok(sub)
    }
}

fn substitute(old: &AST, new: &AST, constraints: &Constraints) -> TypeResult<Constraints> {
    let mut constraints = constraints.clone();
    match constraints.constraints.pop() {
        None => Ok(constraints.clone()),
        Some(Constraint(Expression { ast: left }, Expression { ast: right })) => {
            let left = if left.equal_structure(&old) { new.clone() } else { left };
            let right = if right.equal_structure(&old) { new.clone() } else { right };
            let constraints =
                Constraints::from(Constraint(Expression { ast: left }, Expression { ast: right }));
            unify_link(&constraints, &substitute(old, new, &constraints)?)
        }
        Some(constraint) =>
            unify_link(&Constraints::from(constraint), &substitute(old, new, &constraints)?),
    }
}
