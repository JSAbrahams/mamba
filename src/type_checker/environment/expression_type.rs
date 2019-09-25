use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

use crate::common::position::Position;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::environment::actual_type::ActualType;
use crate::type_checker::type_result::TypeErr;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExpressionTypes {
    pub is_nullable:  bool,
    pub is_mutable:   bool,
    pub actual_types: HashSet<ActualType>
}

impl Display for ExpressionTypes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.is_nullable { "?" } else { "" };
        let mutable = if self.is_mutable { "mut " } else { "" };
        write!(f, "{}{}{}", mutable, fmt_actual_types(&self.actual_types), nullable)
    }
}

fn fmt_actual_types(actual_types: &HashSet<ActualType>) -> String {
    let mut result = String::new();
    actual_types.iter().for_each(|ty| result += format!("{} or ", ty).as_str());
    result.truncate(result.len() - 4);
    result
}

impl ExpressionTypes {
    /// Create new expression type.
    ///
    /// An expression may be nullable or mutable.
    /// Internally also has an actual type, which may either be a type or a
    /// tuple of types.
    pub fn new(types: &[Type]) -> ExpressionTypes {
        ExpressionTypes {
            is_nullable:  false,
            is_mutable:   false,
            actual_types: HashSet::from_iter(vec![ActualType::from(types)].into_iter())
        }
    }

    /// Add a single type to the current type
    ///
    /// May be either a single type, or a tuple of types of multiple types
    /// passed
    pub fn add_type(self, types: &[Type]) -> ExpressionTypes {
        let mut new_actual_types = self.actual_types.clone();
        new_actual_types.insert(ActualType::from(types));

        ExpressionTypes {
            is_nullable:  self.is_nullable,
            is_mutable:   self.is_mutable,
            actual_types: new_actual_types
        }
    }

    /// Remove type from current type
    ///
    /// May be either a single type, or a tuple of types of multiple types
    /// passed.
    ///
    /// # Failure
    ///
    /// If current type does not have passed Type.
    pub fn remove_type(self, types: &[Type], pos: &Position) -> Result<ExpressionTypes, TypeErr> {
        let mut new_actual_types = self.actual_types.clone();
        let actual_type = ActualType::from(types);

        if new_actual_types.contains(&actual_type) {
            new_actual_types.remove(&actual_type);
            Ok(ExpressionTypes {
                is_nullable:  self.is_nullable,
                is_mutable:   self.is_mutable,
                actual_types: new_actual_types
            })
        } else {
            Err(TypeErr::new(pos, "Type does not have type"))
        }
    }

    /// Get the actual type
    ///
    /// # Failure
    ///
    /// If this ExpressionType represents more than one ActualType
    pub fn get_actual_ty(&self, pos: &Position) -> Result<ActualType, TypeErr> {
        if self.actual_types.len() == 1 {
            let first = self.actual_types.clone().into_iter().next();
            Ok(first.unwrap_or_else(|| unreachable!()))
        } else {
            Err(TypeErr::new(pos, "Expected only one type"))
        }
    }

    pub fn union(self, other: &ExpressionTypes) -> ExpressionTypes {
        let union = self.actual_types.union(&other.actual_types);
        ExpressionTypes {
            is_nullable:  self.is_nullable || other.is_nullable,
            is_mutable:   self.is_mutable || other.is_mutable,
            actual_types: union.cloned().collect()
        }
    }

    pub fn mutable(self) -> ExpressionTypes {
        ExpressionTypes {
            is_nullable:  self.is_nullable,
            is_mutable:   true,
            actual_types: self.actual_types
        }
    }

    pub fn nullable(self) -> ExpressionTypes {
        ExpressionTypes {
            is_nullable:  true,
            is_mutable:   self.is_mutable,
            actual_types: self.actual_types
        }
    }
}
