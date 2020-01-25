use crate::type_checker::constraints::constraint::expected::Expected;

pub mod builder;
pub mod expected;
pub mod iterator;

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

    pub fn replace_left(&mut self, new: &Expected) { self.left = new.clone(); }

    pub fn replace_right(&mut self, new: &Expected) { self.right = new.clone(); }

    fn flag(&self) -> Constraint { Constraint { flagged: true, ..self.clone() } }
}
