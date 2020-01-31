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
            let msg = match (&constraint.parent.expect, &constraint.child.expect) {
                (Expect::Type { type_name, .. }, _) | (_, Expect::Type { type_name, .. }) =>
                    format!("Cannot infer type: expected a {}", type_name),
                _ => String::from("Cannot infer type")
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
