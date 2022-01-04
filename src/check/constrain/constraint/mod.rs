use std::fmt::{Display, Error, Formatter};

use crate::check::constrain::constraint::expected::Expect::{Access, Function, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::context::{clss, function};
use crate::check::context::name::{DirectName, NameUnion};

pub mod builder;
pub mod expected;
pub mod iterator;

#[derive(Clone, Debug)]
pub struct Constraint {
    pub is_flag: bool,
    pub is_sub: bool,
    pub msg: String,
    pub parent: Expected,
    pub child: Expected,
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} == {}", self.parent, self.child)
    }
}

impl Constraint {
    pub fn new(msg: &str, parent: &Expected, child: &Expected) -> Constraint {
        Constraint {
            parent: parent.clone(),
            child: child.clone(),
            msg: String::from(msg),
            is_flag: false,
            is_sub: false,
        }
    }

    /// Flag constraint iff flagged is 0, else ignored.
    fn flag(&self) -> Constraint { Constraint { is_flag: true, ..self.clone() } }

    pub fn stringy(msg: &str, expected: &Expected) -> Constraint {
        let string =
            Expected::new(&expected.pos, &Type { name: NameUnion::from(clss::STRING_PRIMITIVE) });
        let access = Access {
            entity: Box::from(expected.clone()),
            name: Box::new(Expected::new(&expected.pos, &Function {
                name: DirectName::from(function::STR),
                args: vec![expected.clone()],
            })),
        };

        Constraint::new(msg, &string, &Expected::new(&expected.pos, &access))
    }

    pub fn truthy(msg: &str, expected: &Expected) -> Constraint {
        let bool =
            Expected::new(&expected.pos, &Type { name: NameUnion::from(clss::BOOL_PRIMITIVE) });
        let access = Access {
            entity: Box::from(expected.clone()),
            name: Box::new(Expected::new(&expected.pos, &Function {
                name: DirectName::from(function::TRUTHY),
                args: vec![expected.clone()],
            })),
        };

        Constraint::new(msg, &bool, &Expected::new(&expected.pos, &access))
    }
}
