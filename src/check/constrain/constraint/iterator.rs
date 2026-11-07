use std::collections::VecDeque;
use std::rc::Rc;

use log::trace;

use crate::check::constrain::constraint::expected::Expected;
use crate::check::constrain::constraint::Constraint;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

/// The constraints of one set, as unification consumes and rewrites them.
#[derive(Debug)]
pub struct Constraints {
    pub(super) constraints: VecDeque<Constraint>,
}

/// One set of constraints from the builder, sharing the constraints it has in common with other sets.
#[derive(Debug)]
pub struct ConstraintSet {
    pub pos: Position,
    pub msg: String,

    pub(super) constraints: Vec<Rc<Constraint>>,
}

impl ConstraintSet {
    pub fn new(pos: Position, msg: String, constraints: Vec<Rc<Constraint>>) -> Self {
        ConstraintSet {
            pos,
            msg,
            constraints,
        }
    }

    /// The constraints to unify, which unification rewrites, so each needs its own copy.
    /// A constraint that no other set still points to is moved instead of copied.
    pub fn into_constraints(self) -> Constraints {
        let constraints = self
            .constraints
            .into_iter()
            .map(|constraint| Rc::try_unwrap(constraint).unwrap_or_else(|shared| (*shared).clone()))
            .collect();
        Constraints { constraints }
    }
}

impl Constraints {
    pub fn len(&self) -> usize {
        self.constraints.len()
    }

    pub fn pop_constr(&mut self) -> Option<Constraint> {
        self.constraints.pop_front()
    }

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
                constraint.msg, constraint.parent.expect, constraint.child.expect
            );
            return Err(vec![TypeErr::new(constraint.parent.pos, &msg)]);
        }

        self.constraints.push_back(constraint.flag());
        Ok(())
    }
}
