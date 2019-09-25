use std::collections::HashSet;

use crate::common::position::Position;
use crate::type_checker::context::concrete::Type;
use crate::type_checker::environment::expression_type::ExpressionTypes;
use crate::type_checker::type_result::TypeErr;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct InferType {
    pub raises: HashSet<Type>,
    expr_type:  Option<ExpressionTypes>
}

impl From<Type> for InferType {
    /// Create new InferType which raises no errors and has given type.
    ///
    /// If multiple types given, type automatically becomes a tuple.
    fn from(ty: Type) -> Self {
        InferType { raises: HashSet::new(), expr_type: Some(ExpressionTypes::new(&[ty])) }
    }
}

impl From<Vec<Type>> for InferType {
    /// Create new InferType which raises no errors and has the given types.
    ///
    /// If multiple types given, type automatically becomes a tuple.
    fn from(types: Vec<Type>) -> Self {
        InferType { raises: HashSet::new(), expr_type: Some(ExpressionTypes::new(&types)) }
    }
}

impl InferType {
    /// Create new InferType without errors or expression type.
    ///
    /// This represents a statement.
    pub fn new() -> InferType { InferType { raises: HashSet::new(), expr_type: None } }

    /// Create union of this type and other type
    ///
    /// Errors raised are joined.
    ///
    /// # Failure
    ///
    /// If one type represents a statement and another an expression
    pub fn union(self, other: &InferType, pos: &Position) -> Result<InferType, TypeErr> {
        let union = self.raises.union(&other.raises);
        Ok(InferType {
            raises:    union.cloned().collect(),
            expr_type: match (&self.expr_type, &other.expr_type) {
                (None, None) => None,
                (Some(self_ty), Some(other_ty)) => Some(self_ty.clone().union(other_ty)),
                _ => return Err(TypeErr::new(pos, "Types are incompatible"))
            }
        })
    }

    pub fn mutable(self, pos: &Position) -> Result<Self, TypeErr> {
        Ok(InferType {
            raises:    self.raises,
            expr_type: Some(
                self.expr_type.ok_or(TypeErr::new(pos, "Is not an expression"))?.mutable()
            )
        })
    }

    pub fn nullable(self, pos: &Position) -> Result<Self, TypeErr> {
        Ok(InferType {
            raises:    self.raises,
            expr_type: Some(
                self.expr_type.ok_or(TypeErr::new(pos, "Is not an expression"))?.nullable()
            )
        })
    }

    pub fn is_mutable(&self, pos: &Position) -> Result<bool, TypeErr> {
        Ok(self.expr_type.clone().ok_or(TypeErr::new(pos, "Is not an expression"))?.is_mutable)
    }

    pub fn is_nullable(&self, pos: &Position) -> Result<bool, TypeErr> {
        Ok(self.expr_type.clone().ok_or(TypeErr::new(pos, "Is not an expression"))?.is_nullable)
    }

    /// Get expression type
    ///
    /// # Failure
    ///
    /// If a statement type
    pub fn expr_tys(&self, pos: &Position) -> Result<ExpressionTypes, TypeErr> {
        self.expr_type.clone().ok_or(TypeErr::new(pos, "Is not an expression"))
    }

    /// Add errors to type
    pub fn raises(self, raises: HashSet<Type>) -> InferType {
        let mut self_raises = self.raises.clone();
        raises.iter().for_each(|err| {
            self_raises.insert(err.clone());
        });

        InferType { raises: self_raises, expr_type: self.expr_type }
    }

    /// Remove errors from type
    ///
    /// # Failure
    ///
    /// If attempting to remove error which is not there.
    pub fn handled(
        self,
        handled: HashSet<Type>,
        pos: &Position
    ) -> Result<InferType, Vec<TypeErr>> {
        let mut self_raises = self.raises.clone();
        let mut errors = vec![];
        handled.iter().for_each(|handled| {
            if self_raises.contains(handled) {
                self_raises.remove(handled);
            } else {
                errors.push(TypeErr::new(pos, "Type does not have error"))
            }
        });

        if errors.is_empty() {
            Ok(InferType { raises: self_raises, expr_type: self.expr_type })
        } else {
            Err(errors)
        }
    }
}
