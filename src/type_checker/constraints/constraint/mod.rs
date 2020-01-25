use crate::type_checker::constraints::constraint::expected::Expected;

pub mod builder;
pub mod expected;
pub mod iterator;

#[derive(Clone, Debug)]
pub struct Constraint {
    pub flagged: bool,
    pub parent:  Expected,
    pub child:   Expected
}

impl Constraint {
    pub fn new(left: Expected, right: Expected) -> Constraint {
        Constraint { parent: left, child: right, flagged: false }
    }

    pub fn replace_left(&mut self, new: &Expected) { self.parent = new.clone(); }

    pub fn replace_right(&mut self, new: &Expected) { self.child = new.clone(); }

    fn flag(&self) -> Constraint { Constraint { flagged: true, ..self.clone() } }
}
