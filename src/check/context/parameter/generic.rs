use std::fmt::{Display, Error, Formatter};

use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GenericParameter {
    pub is_py_type: bool,
    pub name: StringName,
    pub parent: Option<TrueName>,
}

impl Display for GenericParameter {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}{}",
            self.name,
            if let Some(parent) = &self.parent {
                format!(" isa {parent}")
            } else {
                String::new()
            }
        )
    }
}
