use crate::common::position::Position;
use crate::parser::ast::Node::Bool;
use crate::parser::ast::AST;
use crate::type_checker::constraints::cons::Expect::*;
use crate::type_checker::constraints::cons::{Constraints, Expected};
use crate::type_checker::type_result::{TypeErr, TypeResult};

/// Empties out constraints and puts them in a substituted list.
pub fn unify_link(constr: &Constraints, sub: &Constraints) -> TypeResult<Constraints> {
    let mut constraints = constr.clone();
    let mut sub = sub.clone();

    while let Some(constraint) = constraints.constraints.pop() {
        let unified = match (&constraint.0.expect, &constraint.1.expect) {
            (ExpressionAny, ExpressionAny) | (Truthy, Truthy) | (RaisesAny, RaisesAny) =>
                Ok(Constraints::from(&constraint)),

            (Expression { .. }, ExpressionAny) | (ExpressionAny, Expression { .. }) =>
                Ok(Constraints::from(&constraint)),
            (Type { .. }, ExpressionAny) | (ExpressionAny, Type { .. }) =>
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

            (Truthy, Expression { ast: AST { node: Bool { .. }, .. } })
            | (Expression { ast: AST { node: Bool { .. }, .. } }, Truthy) =>
                Ok(Constraints::from(&constraint)),

            (Type { type_name }, Implements { type_name: f_name, args })
            | (Implements { type_name: f_name, args }, Type { type_name }) => unimplemented!(),

            _ => panic!(
                "Unexpected: {}={} : {:?} == {:?}",
                constraint.0.pos, constraint.1.pos, constraint.1.expect, constraint.1.expect
            )
        }?;

        sub.append(&unified);
    }

    Ok(sub)
}

fn substitute(
    old: &Expected,
    new: &Expected,
    constraints: &Constraints
) -> TypeResult<Constraints> {
    let mut constraints = constraints.clone();
    if let Some(constraint) = constraints.constraints.pop() {
        let (left, right) = (constraint.0, constraint.1);
        let left =
            if left.expect == old.expect { Expected::new(&left.pos, &new.expect) } else { left };
        let right =
            if right.expect == old.expect { Expected::new(&right.pos, &new.expect) } else { right };

        let mut unified = Constraints::new().add(&left, &right);
        Ok(unified.append(&substitute(old, new, &constraints)?))
    } else {
        Ok(constraints.clone())
    }
}
