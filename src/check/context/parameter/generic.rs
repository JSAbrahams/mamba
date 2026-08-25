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

#[cfg(test)]
mod test {
    use super::GenericParameter;
    use crate::check::name::string_name::StringName;
    use crate::check::name::true_name::TrueName;

    #[test]
    fn display_without_parent() {
        let param = GenericParameter {
            is_py_type: true,
            name: StringName::from("T"),
            parent: None,
        };
        assert_eq!(param.to_string(), "T");
    }

    #[test]
    fn display_with_parent() {
        let param = GenericParameter {
            is_py_type: true,
            name: StringName::from("T"),
            parent: Some(TrueName::from(&StringName::from("Comparable"))),
        };
        assert_eq!(param.to_string(), "T isa Comparable");
    }
}
