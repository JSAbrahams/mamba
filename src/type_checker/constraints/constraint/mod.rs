use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::collections::VecDeque;

pub mod expected;

#[derive(Clone, Debug)]
pub struct Constraints {
    pub constraints: VecDeque<Constraint>
}

impl Constraints {
    pub fn new() -> Constraints { Constraints { constraints: VecDeque::new() } }

    pub fn add(&self, left: &Expected, right: &Expected) -> Constraints {
        let mut new_constr = self.constraints.clone();
        new_constr.push_back(Constraint::new(left.clone(), right.clone()));
        Constraints { constraints: new_constr }
    }

    pub fn append(&mut self, constraints: &Constraints) {
        self.constraints.append(&mut constraints.constraints.clone());
    }

    pub fn push(&mut self, left: &Expected, right: &Expected) {
        self.constraints.push_back(Constraint::new(left.clone(), right.clone()))
    }

    pub fn reinsert(&mut self, constraint: &Constraint) -> TypeResult<()> {
        if constraint.flagged {
            return Err(vec![TypeErr::new(&constraint.left.pos, "Cannot infer type.")]);
        }

        self.constraints.push_back(constraint.flag());
        Ok(())
    }

    pub fn pop_constr(&mut self) -> Option<Constraint> { self.constraints.pop_front() }
}

impl From<&Constraint> for Constraints {
    fn from(constraint: &Constraint) -> Self {
        Constraints { constraints: VecDeque::from(vec![constraint.clone()]) }
    }
}

#[derive(Clone, Debug)]
pub struct Constraint {
    pub flagged: bool,
    pub left:    Expected,
    pub right:   Expected
}

impl Constraint {
    pub fn new(left: Expected, right: Expected) -> Constraint {
        Constraint { left, right, flagged: false }
    }

    fn flag(&self) -> Constraint { Constraint { flagged: true, ..self.clone() } }
}
