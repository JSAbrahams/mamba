use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::Constraint;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::VecDeque;

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

    pub fn push_constr(&mut self, constr: &Constraint) {
        self.constraints.push_back(constr.clone())
    }

    pub fn push(&mut self, left: &Expected, right: &Expected) {
        self.constraints.push_back(Constraint::new(left.clone(), right.clone()))
    }

    pub fn reinsert(&mut self, constraint: &Constraint) -> TypeResult<()> {
        if constraint.flagged {
            // Can only reinsert constraint once
            return Err(vec![TypeErr::new(&constraint.left.pos, "Cannot infer type.")]);
        }

        self.constraints.push_back(constraint.flag());
        Ok(())
    }
}

impl Default for Constraints {
    fn default() -> Self { Constraints::new(&[]) }
}
