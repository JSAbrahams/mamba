use crate::type_checker::constraints::constraint::expected::Expected;

pub mod builder;
pub mod expected;
pub mod iterator;

#[derive(Clone, Debug)]
pub struct Constraint {
    pub is_flag: bool,
    pub is_sub:  bool,
    pub idents:  Vec<String>,
    pub parent:  Expected,
    pub child:   Expected
}

impl Constraint {
    pub fn new(left: &Expected, right: &Expected) -> Constraint {
        Constraint {
            parent:  left.clone(),
            child:   right.clone(),
            idents:  vec![],
            is_flag: false,
            is_sub:  false
        }
    }

    /// Flag constraint iff flagged is 0, else ignored.
    fn flag(&self) -> Constraint { Constraint { is_flag: true, ..self.clone() } }
}
