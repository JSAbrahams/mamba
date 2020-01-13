use crate::parser::ast::AST;
use crate::type_checker::constraints::cons::Expect::*;
use crate::type_checker::constraints::cons::{Constraint, Constraints, Expect};
use crate::type_checker::type_result::TypeResult;
use std::ops::Deref;

/// Empties out constraints and puts them in a substituted list.
pub fn unify_link(
    left: Option<&Expect>,
    right: Option<&Expect>,
    constr: &mut Constraints,
    sub: Constraints
) -> TypeResult<Constraints> {
    let (left, right) = if let (Some(left), Some(right)) = (left, right) {
        (left.clone(), right.clone())
    } else if let Some(Constraint(left, right)) = constr.constraints.pop() {
        (left, right)
    } else {
        return Ok(constr.clone());
    };

    match (&left, &right) {
        (ExpressionAny, ExpressionAny) | (Truthy, Truthy) | (RaisesAny, RaisesAny) =>
            Ok(sub.add(&left, &right)),

        (Nullable { expect: left }, Nullable { expect: right })
        | (Collection { ty: left }, Collection { ty: right })
        | (Mutable { expect: left }, Mutable { expect: right }) =>
            unify_link(Some(left.deref()), Some(right.deref()), constr, sub),

        (Expression { ast: left }, Expression { ast: right }) => substitute(&left, &right, &sub),

        (Raises { type_name: l_name }, Raises { type_name: r_name }) if l_name == r_name =>
            Ok(sub.add(&left, &right)),
        (Raises { .. }, Raises { .. }) => unimplemented!(),

        (Implements { type_name: ltn, args: la }, Implements { type_name: rtn, args: ra })
            if ltn == rtn && la.len() == ra.len() =>
        {
            for (la, ra) in la.into_iter().zip(ra) {
                unify_link(Some(la), Some(ra), constr, sub.clone())?;
            }
            unimplemented!()
        }
        (Implements { .. }, Implements { .. }) => unimplemented!(),

        (HasField { name: _ }, HasField { name: _ }) => unimplemented!(),

        _ => unimplemented!()
    }
}

fn substitute(_: &AST, _: &AST, _: &Constraints) -> TypeResult<Constraints> { unimplemented!() }
