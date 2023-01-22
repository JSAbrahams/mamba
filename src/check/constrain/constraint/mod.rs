use std::fmt::{Display, Error, Formatter};

use crate::check::constrain::constraint::builder::VarMapping;
use crate::check::constrain::constraint::expected::Expect::{Access, Function, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::context::clss;
use crate::check::context::function::python::{STR, TRUTHY};
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
    pub parent: Expected,
    pub child: Expected,
}

pub(super) trait MapExp {
    fn map_exp(&self, var_mapping: &VarMapping, global_var_mapping: &VarMapping) -> Self;
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{} >= {}", self.parent, self.child)
    }
}

impl MapExp for Constraint {
    fn map_exp(&self, var_mapping: &VarMapping, global_var_mapping: &VarMapping) -> Self {
        let left = self.parent.map_exp(var_mapping, global_var_mapping);
        let right = self.child.map_exp(var_mapping, global_var_mapping);
        Constraint { parent: left, child: right, ..self.clone() }
    }
}

impl Constraint {
    /// Create new constraint.
    ///
    /// By default, the left side is assumed to be the superset of the right side.
    pub fn new(msg: &str, parent: &Expected, child: &Expected) -> Constraint {
        let (parent, child) = (parent.clone(), child.clone());
        Constraint { parent, child, msg: String::from(msg), is_flag: false, is_sub: false }
    }

    /// Flag constraint iff flagged is 0, else ignored.
    fn flag(&self) -> Constraint { Constraint { is_flag: true, ..self.clone() } }

    pub fn stringy(msg: &str, expected: &Expected) -> Constraint {
        Self::access(msg, expected, &Name::from(clss::STRING), &StringName::from(STR))
    }

    pub fn truthy(msg: &str, expected: &Expected) -> Constraint {
        Self::access(msg, expected, &Name::from(clss::BOOL), &StringName::from(TRUTHY))
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
