use std::collections::VecDeque;

use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::constraints::constraint::expected::{Expect, Expected};
use crate::type_checker::constraints::constraint::Constraint;

#[derive(Clone, Debug)]
pub struct Constraints {
    constraints: VecDeque<Constraint>
}

impl Constraints {
    pub fn new(constraints: &[Constraint]) -> Constraints {
        Constraints { constraints: VecDeque::from(Vec::from(constraints)) }
    }

    pub fn len(&self) -> usize { self.constraints.len() }

    pub fn pop_constr(&mut self) -> Option<Constraint> { self.constraints.pop_front() }

    pub fn append(&mut self, constraints: &Constraints) {
        self.constraints.append(&mut constraints.constraints.clone());
    }

    pub fn eager_push(&mut self, left: &Expected, right: &Expected) {
        self.constraints.push_front(Constraint::new(left, right))
    }

    pub fn push_constr(&mut self, constr: &Constraint) {
        self.constraints.push_back(constr.clone())
    }

    pub fn push(&mut self, left: &Expected, right: &Expected) {
        self.constraints.push_back(Constraint::new(left, right))
    }

    pub fn reinsert(&mut self, constraint: &Constraint) -> TypeResult<()> {
        if constraint.flagged {
            // Can only reinsert constraint once
            // TODO create pretty print for asts
            let msg = match (&constraint.parent.expect, &constraint.child.expect) {
                (Expect::Expression { .. }, Expect::Expression { .. }) =>
                    String::from("Cannot infer type"),
                (other, Expect::Expression { ast }) | (Expect::Expression { ast }, other) =>
                    format!("Cannot infer type: {} and {:?}", other, ast),
                _ => String::from("cannot infer type")
            };

            return Err(vec![TypeErr::new(&constraint.parent.pos, &msg)]);
        }

        self.constraints.push_back(constraint.flag());
        Ok(())
    }
}

impl Default for Constraints {
    fn default() -> Self { Constraints::new(&[]) }
}
