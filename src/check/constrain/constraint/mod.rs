use std::fmt::{Display, Error, Formatter};

use crate::check::constrain::constraint::expected::Expect::{Access, Function, Type};
use crate::check::constrain::constraint::expected::Expected;
use crate::check::context::name::{DirectName, NameUnion};
use crate::check::context::{clss, function};
use crate::common::delimit::{comma_delm, custom_delimited};

pub mod builder;
pub mod expected;
pub mod iterator;

#[derive(Clone, Debug)]
pub struct Constraint {
    pub is_flag:    bool,
    pub is_sub:     bool,
    pub is_gen:     bool,
    pub is_flipped: bool,
    pub ids:        Vec<String>,
    pub msg:        String,
    pub parent:     Expected,
    pub child:      Expected
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let is_flag = if self.is_flag { "(flagged)" } else { "" };
        let is_sub = if self.is_sub { "(sub)" } else { "" };
        let idents = if self.ids.is_empty() {
            String::new()
        } else {
            format!("(ids: {})", comma_delm(&self.ids))
        };
        let is_gen = if self.is_gen { "(gen)" } else { "" };

        let flags = custom_delimited(
            vec![is_flag, is_sub, is_gen, idents.as_str()].drain_filter(|f| !f.is_empty()),
            " ",
            ""
        );
        let flags = if flags.is_empty() { String::new() } else { format!("{{{}}} # ", flags) };

        let parent = if self.parent.is_expr() {
            format!("{}", self.parent)
        } else {
            format!("[{}]", self.parent)
        };
        let eq = if self.is_flipped { "<=" } else { ">=" };
        let child = if self.child.is_expr() {
            format!("{}", self.child)
        } else {
            format!("[{}]", self.child)
        };

        write!(f, "{}{} {} {}", flags, parent, eq, child)
    }
}

impl Constraint {
    pub fn new(msg: &str, parent: &Expected, child: &Expected) -> Constraint {
        Constraint {
            parent:     parent.clone(),
            child:      child.clone(),
            is_flipped: false,
            is_gen:     false,
            msg:        String::from(msg),
            ids:        vec![],
            is_flag:    false,
            is_sub:     false
        }
    }

    /// Flag constraint iff flagged is 0, else ignored.
    fn flag(&self) -> Constraint { Constraint { is_flag: true, ..self.clone() } }

    fn as_gen(&self) -> Constraint { Constraint { is_gen: true, ..self.clone() } }

    pub fn stringy(msg: &str, expected: &Expected) -> Constraint {
        let string =
            Expected::new(&expected.pos, &Type { name: NameUnion::from(clss::STRING_PRIMITIVE) });
        let access = Access {
            entity: Box::from(expected.clone()),
            name:   Box::new(Expected::new(&expected.pos, &Function {
                name: DirectName::from(function::STR),
                args: vec![expected.clone()]
            }))
        };

        Constraint::new(msg, &string, &Expected::new(&expected.pos, &access))
    }

    pub fn truthy(msg: &str, expected: &Expected) -> Constraint {
        let bool =
            Expected::new(&expected.pos, &Type { name: NameUnion::from(clss::BOOL_PRIMITIVE) });
        let access = Access {
            entity: Box::from(expected.clone()),
            name:   Box::new(Expected::new(&expected.pos, &Function {
                name: DirectName::from(function::TRUTHY),
                args: vec![expected.clone()]
            }))
        };

        Constraint::new(msg, &bool, &Expected::new(&expected.pos, &access))
    }
}
