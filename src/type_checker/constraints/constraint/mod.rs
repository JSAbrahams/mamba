use crate::type_checker::constraints::constraint::expected::Expected;

pub mod builder;
pub mod expected;
pub mod iterator;

#[derive(Clone, Debug)]
pub struct Constraint {
    pub flagged:     bool,
    pub substituted: bool,
    pub identifiers: Vec<String>,
    pub parent:      Expected,
    pub child:       Expected
}

impl Constraint {
    pub fn new(left: &Expected, right: &Expected) -> Constraint {
        Constraint {
            parent:      left.clone(),
            child:       right.clone(),
            identifiers: vec![],
            flagged:     false,
            substituted: false
        }
    }

    fn flag(&self) -> Constraint { Constraint { flagged: true, ..self.clone() } }
}
