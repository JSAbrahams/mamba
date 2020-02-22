use std::fmt::{Display, Error, Formatter};

use crate::check::constrain::constraint::expected::Expected;
use crate::common::delimit::comma_delm;

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

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let is_flag = if self.is_flag { "(flagged) " } else { "" };
        let is_sub = if self.is_flag { "(subs) " } else { "" };
        let idents = if self.idents.is_empty() {
            String::new()
        } else {
            format!("(idents: {}) ", comma_delm(&self.idents))
        };
        write!(f, "{}{}{}{} = {}", is_flag, is_sub, idents, self.parent, self.child)
    }
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
