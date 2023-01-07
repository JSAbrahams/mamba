use std::collections::VecDeque;

use crate::check::constrain::constraint::Constraint;
use crate::check::constrain::constraint::expected::Expected;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

#[derive(Clone, Debug)]
pub struct Constraints {
    pub pos: Position,
    pub msg: String,

    pub(in super) constraints: VecDeque<Constraint>,
}

impl From<(Position, String, Vec<Constraint>)> for Constraints {
    fn from((pos, msg, constraints): (Position, String, Vec<Constraint>)) -> Self {
        Constraints { pos, msg, constraints: VecDeque::from(constraints) }
    }
}

impl Constraints {
    pub fn len(&self) -> usize { self.constraints.len() }

    pub fn pop_constr(&mut self) -> Option<Constraint> { self.constraints.pop_front() }

    /// Push constraint at front so that it will be analysed next.
    ///
    /// Only used during unification stage.
    pub fn push(&mut self, msg: &str, left: &Expected, right: &Expected) {
        let constraint = Constraint::new(msg, left, right);
        trace!("{:width$}[gen {}] {}", "", msg, constraint, width = 17);
        self.constraints.push_front(constraint)
    }

    pub fn push_constr(&mut self, constr: &Constraint) {
        self.constraints.push_back(constr.clone())
    }

    pub fn reinsert(&mut self, constraint: &Constraint) -> TypeResult<()> {
        if constraint.is_flag {
            // Can only reinsert constraint once
            let msg = format!(
                "Cannot infer type within {}. Expected a {}, was {}",
                constraint.msg, &constraint.left.expect, &constraint.right.expect
            );
            return Err(vec![TypeErr::new(constraint.left.pos, &msg)]);
        }

        self.constraints.push_back(constraint.flag());
        Ok(())
    }
}
