use std::collections::HashSet;

use crate::common::position::Position;
use crate::type_checker::context::type_name::concrete::actual::ActualTypeName;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use std::iter::FromIterator;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct InferType {
    pub raises: HashSet<ActualTypeName>,
    expr_type:  Option<ExpressionType>
}

impl From<&ExpressionType> for InferType {
    fn from(expr_type: &ExpressionType) -> Self {
        InferType { raises: HashSet::new(), expr_type: Some(expr_type.clone()) }
    }
}

impl InferType {
    pub fn new() -> InferType { InferType { raises: HashSet::new(), expr_type: None } }

    pub fn union(self, other: &InferType, pos: &Position) -> Result<InferType, TypeErr> {
        Ok(InferType {
            raises:    self.raises.union(&other.raises).cloned().collect(),
            expr_type: match (&self.expr_type, &other.expr_type) {
                (None, None) => None,
                (Some(self_ty), Some(other_ty)) => Some(self_ty.clone().union(other_ty)),
                _ => return Err(TypeErr::new(pos, "Types are incompatible"))
            }
        })
    }

    pub fn expr_ty(&self, pos: &Position) -> Result<ExpressionType, TypeErr> {
        self.expr_type.clone().ok_or(TypeErr::new(pos, "Is not an expression"))
    }

    pub fn add_raise_from_type(self, raised: InferType, pos: &Position) -> TypeResult<InferType> {
        let expr_ty = raised.expr_ty(pos)?;
        let raises = match expr_ty {
            ExpressionType::Single { mut_ty } =>
                HashSet::from_iter(vec![ActualTypeName::from(mut_ty.actual_ty)].to_iter()),
            ExpressionType::Union { union } => union.iter().map(|mut_ty| mut_ty.actual_ty).collect()
        };
        Ok(self.add_raises(raises))
    }

    pub fn add_raises(self, raises: HashSet<ActualTypeName>) -> InferType {
        let raises = self.raises.union(&raises).into_iter().collect();
        InferType { raises, expr_type: self.expr_type }
    }

    pub fn handled(
        self,
        handled: HashSet<ActualTypeName>,
        pos: &Position
    ) -> TypeResult<InferType> {
        let mut self_raises = self.raises.clone();
        let mut errors = vec![];
        handled.into_iter().for_each(|handled| {
            if self_raises.contains(&handled) {
                self_raises.remove(&handled);
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
