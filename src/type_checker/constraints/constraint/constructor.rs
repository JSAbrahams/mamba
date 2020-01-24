use crate::type_checker::constraints::constraint::expected::Expected;
use crate::type_checker::constraints::constraint::iterator::Constraints;
use crate::type_checker::constraints::constraint::Constraint;

#[derive(Clone, Debug)]
pub struct ConstraintConstructor {
    inner: bool,
    pub inner_constr: Vec<Vec<Constraint>>,
    current_inner: Vec<Constraint>,
    outer_constr: Vec<Constraint>
}

impl ConstraintConstructor {
    pub fn new() -> ConstraintConstructor {
        ConstraintConstructor {
            inner:         false,
            inner_constr:  vec![],
            current_inner: vec![],
            outer_constr:  vec![]
        }
    }

    pub fn switch_to_inner(&mut self) { self.inner = true }

    pub fn switch_to_outer(&mut self) {
        if !self.current_inner.is_empty() {
            self.inner_constr.push(self.current_inner.clone());
        }

        self.inner = false;
        self.current_inner.clear();
    }

    pub fn add(&mut self, left: &Expected, right: &Expected) {
        if self.inner {
            self.current_inner.push(Constraint::new(left.clone(), right.clone()));
        } else {
            self.outer_constr.push(Constraint::new(left.clone(), right.clone()));
        }
    }

    pub fn all_constr(self) -> Vec<Constraints> {
        let mut all_constraints = self.inner_constr.clone();
        if !self.current_inner.is_empty() {
            all_constraints.push(self.outer_constr.clone());
        }
        if !self.outer_constr.is_empty() {
            all_constraints.push(self.outer_constr.clone());
        }

        all_constraints.iter().map(|constraints| Constraints::new(constraints)).collect()
    }
}
