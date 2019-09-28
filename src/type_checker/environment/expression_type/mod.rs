use std::collections::HashSet;
use std::iter::FromIterator;

use crate::common::position::Position;
use crate::type_checker::environment::expression_type::mutable_type::MutableType;
use crate::type_checker::type_result::TypeErr;

mod actual_type;
mod mutable_type;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ExpressionType {
    Single { expr_ty: MutableType },
    Union { union: HashSet<MutableType> }
}

impl ExpressionType {
    pub fn union(self, other: &ExpressionType) -> ExpressionType {
        match (self, other) {
            (ExpressionType::Single { expr_ty }, ExpressionType::Single { expr_ty: other }) =>
                ExpressionType::Union { union: HashSet::from_iter(vec![expr_ty, other].iter()) },
            (ExpressionType::Single { expr_ty }, ExpressionType::Union { union })
            | (ExpressionType::Union { union }, ExpressionType::Single { expr_ty }) => {
                let mut union = union.clone();
                union.insert(expr_ty);
                ExpressionType::Union { union }
            }
            (ExpressionType::Union { union }, ExpressionType::Union { union: other }) => {
                let mut union = union.clone();
                union.extend(other);
                ExpressionType::Union { union }
            }
        }
    }

    pub fn field(&self, name: &str, field: &str, pos: &Position) -> Result<Option<Field>, TypeErr> {
        unimplemented!()
    }

    pub fn fun(
        &self,
        name: &str,
        args: &[ExpressionType],
        pos: &Position
    ) -> Result<Option<Function>, TypeErr> {
        unimplemented!()
    }
}
