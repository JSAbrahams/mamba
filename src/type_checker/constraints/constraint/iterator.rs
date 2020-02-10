use std::collections::VecDeque;

use crate::type_checker::checker_result::{TypeErr, TypeResult};
use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::Constraint;
use crate::type_checker::ty_name::TypeName;

#[derive(Clone, Debug)]
pub struct Constraints {
    pub in_class: Vec<TypeName>,
    constraints:  VecDeque<Constraint>
}

impl Constraints {
    pub fn new(constraints: &[Constraint], in_class: &[TypeName]) -> Constraints {
        Constraints {
            in_class:    Vec::from(in_class),
            constraints: VecDeque::from(Vec::from(constraints))
        }
    }

    pub fn len(&self) -> usize { self.constraints.len() }

    pub fn pop_constr(&mut self) -> Option<Constraint> { self.constraints.pop_front() }

    pub fn eager_push_constr(&mut self, constr: &Constraint) {
        self.constraints.push_front(constr.clone())
    }

    pub fn eager_push(&mut self, left: &Expected, right: &Expected) {
        self.constraints.push_front(Constraint::new(left, right))
    }

    pub fn push_constr(&mut self, constr: &Constraint) {
        self.constraints.push_back(constr.clone())
    }

    pub fn reinsert(&mut self, constraint: &Constraint) -> TypeResult<()> {
        if constraint.flagged {
            // Can only reinsert constraint once
            let msg = format!(
                "Cannot infer type, expected {} but was {}",
                &constraint.parent.expect, &constraint.child.expect
            );
            return Err(vec![TypeErr::new(&constraint.parent.pos, &msg)]);
        }

        self.constraints.push_back(constraint.flag());
        Ok(())
    }
}

impl Default for Constraints {
    fn default() -> Self { Constraints::new(&[], &[]) }
}
