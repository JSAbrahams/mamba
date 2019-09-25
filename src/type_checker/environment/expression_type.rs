use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

use crate::common::position::Position;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::environment::actual_type::ActualType;
use crate::type_checker::type_result::TypeErr;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExpressionType {
    pub nullable:     bool,
    pub mutable:      bool,
    pub actual_types: HashSet<ActualType>
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.nullable { "?" } else { "" };
        let mutable = if self.mutable { "mut " } else { "" };
        write!(f, "{}{}{}", mutable, self.actual_types, nullable)
    }
}

impl ExpressionType {
    /// Create new expression type.
    ///
    /// An expression may be nullable or mutable.
    /// Internally also has an actual type, which may either be a type or a
    /// tuple of types.
    pub fn new(types: &[Type]) -> ExpressionType {
        ExpressionType {
            nullable:     false,
            mutable:      false,
            actual_types: HashSet::from_iter(ActualType::from(types))
        }
    }

    /// Add a single type to the current type
    ///
    /// May be either a single type, or a tuple of types of multiple types
    /// passed
    pub fn add_type(self, types: &[Type]) -> ExpressionType {
        let mut new_actual_types = self.actual_types.clone();
        new_actual_types.insert(ActualType::from(types));

        ExpressionType {
            nullable:     self.nullable,
            mutable:      self.mutable,
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
    pub fn remove_type(self, types: &[Type], pos: &Position) -> Result<ExpressionType, TypeErr> {
        let mut new_actual_types = self.actual_types.clone();
        let actual_type = ActualType::from(types);

        if new_actual_types.contains(&actual_type) {
            new_actual_types.remove(&actual_type);
            Ok(ExpressionType {
                nullable:     self.nullable,
                mutable:      self.mutable,
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
    pub fn get_actual_type(&self, pos: &Position) -> Result<ActualType, TypeErr> {
        if self.actual_types.size() == 1 {
            Ok(self.actual_types.clone().drain().collect()[0])
        } else {
            Err(TypeErr::new(pos, "Expected only one type"))
        }
    }

    pub fn union(self, other: &ExpressionType) -> ExpressionType {
        let mut actual_types = self.actual_types;
        actual_types.append(other.actual_types.clone());

        ExpressionType {
            nullable: self.nullable || other.nullable,
            mutable: self.mutable || other.mutable,
            actual_types
        }
    }

    pub fn mutable(self) -> ExpressionType {
        ExpressionType {
            nullable:     self.nullable,
            mutable:      true,
            actual_types: self.actual_types
        }
    }

    pub fn nullable(self) -> ExpressionType {
        ExpressionType {
            nullable:     true,
            mutable:      self.mutable,
            actual_types: self.actual_types
        }
    }
}
