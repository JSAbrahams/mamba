use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::infer_type::expression::ExpressionType;
use crate::type_checker::type_name::actual::ActualTypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};
use crate::type_checker::util::comma_delimited;

// TODO make these private
pub mod actual;
pub mod expression;
pub mod nullable;

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

impl Display for InferType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            if let Some(e_ty) = &self.expr_type { format!("{}", e_ty) } else { String::from("()") },
            if self.raises.is_empty() {
                String::new()
            } else {
                format!("raises [{}]", comma_delimited(&self.raises))
            }
        )
    }
}

impl Default for InferType {
    fn default() -> Self { InferType { raises: HashSet::new(), expr_type: None } }
}

impl InferType {
    pub fn is_stmt(&self) -> bool { self.expr_type.is_none() }

    // TODO make union error if union between statement and expression
    pub fn union(self, other: &InferType, pos: &Position) -> Result<InferType, TypeErr> {
        Ok(InferType {
            raises:    self.raises.union(&other.raises).cloned().collect(),
            expr_type: match (&self.expr_type, &other.expr_type) {
                (None, None) => None,
                (Some(self_ty), Some(other_ty)) => Some(self_ty.clone().union(other_ty)),
                (Some(_), None) =>
                    return Err(TypeErr::new(
                        pos,
                        "Cannot make union between expression and statement"
                    )),
                (None, Some(_)) =>
                    return Err(TypeErr::new(
                        pos,
                        "Cannot make union between statement and expression"
                    )),
            }
        })
    }

    pub fn expr_ty(&self, pos: &Position) -> Result<ExpressionType, TypeErr> {
        self.expr_type.clone().ok_or_else(|| TypeErr::new(pos, "Is not an expression"))
    }

    pub fn union_raises(self, raises: &HashSet<ActualTypeName>) -> InferType {
        let raises = self.raises.union(&raises).cloned().collect();
        InferType { raises, ..self }
    }

    pub fn add_raises(self, infer_type: &InferType) -> InferType {
        let raises = self.raises.union(&infer_type.raises).cloned().collect();
        InferType { raises, ..self }
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
