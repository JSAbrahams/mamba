use std::fmt::{Display, Error, Formatter};

use crate::check::constrain::constraint::expected::Expect::{Access, Function, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::context::{clss, function};
use crate::check::name::Name;
use crate::check::name::string_name::StringName;

pub mod builder;
pub mod expected;
pub mod iterator;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Constraint {
    pub is_flag: bool,
    pub is_sub: bool,
    pub msg: String,
    pub left: Expected,
    pub right: Expected,
    pub superset: ConstrVariant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConstrVariant {
    Left,
    Right,
}

impl Default for ConstrVariant {
    fn default() -> Self {
        ConstrVariant::Left
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let superset = match &self.superset {
            ConstrVariant::Left => ">=",
            ConstrVariant::Right => "<="
        };

        write!(f, "{} {superset} {}", self.left, self.right)
    }
}

impl Constraint {
    /// Create new constraint.
    ///
    /// By default, the left side is assumed to be the superset of the right side.
    pub fn new(msg: &str, parent: &Expected, child: &Expected) -> Constraint {
        Constraint::new_variant(msg, parent, child, ConstrVariant::default())
    }

    pub fn new_variant(msg: &str, parent: &Expected, child: &Expected, superset: ConstrVariant)
                       -> Constraint {
        Constraint {
            left: parent.clone(),
            right: child.clone(),
            msg: String::from(msg),
            is_flag: false,
            is_sub: false,
            superset: superset.clone(),
        }
    }

    /// Flag constraint iff flagged is 0, else ignored.
    fn flag(&self) -> Constraint { Constraint { is_flag: true, ..self.clone() } }

    pub fn stringy(msg: &str, expected: &Expected) -> Constraint {
        Self::access(msg, expected, &Name::from(clss::STRING), &StringName::from(function::STR))
    }

    pub fn truthy(msg: &str, expected: &Expected) -> Constraint {
        Self::access(msg, expected, &Name::from(clss::BOOL), &StringName::from(function::TRUTHY))
    }

    fn access(msg: &str, expected: &Expected, ty_name: &Name, fun_name: &StringName) -> Constraint {
        let obj = Expected::new(expected.pos, &Type { name: ty_name.clone() });
        let fun = Function { name: fun_name.clone(), args: vec![expected.clone()] };
        let access = Access {
            entity: Box::from(expected.clone()),
            name: Box::new(Expected::new(expected.pos, &fun)),
        };

        Constraint::new(msg, &obj, &Expected::new(expected.pos, &access))
    }

    pub fn undefined(msg: &str, expected: &Expected) -> Constraint {
        let none = Expected::new(expected.pos, &Type { name: Name::from(clss::NONE) });
        Constraint::new(msg, expected, &none)
    }
}
