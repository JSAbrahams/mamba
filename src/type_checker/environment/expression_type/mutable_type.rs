use std::fmt;
use std::fmt::{Display, Formatter};

use crate::common::position::Position;
use crate::type_checker::context::field::concrete::Field;
use crate::type_checker::context::function::concrete::Function;
use crate::type_checker::context::type_name::concrete::actual::ActualTypeName;
use crate::type_checker::context::type_name::concrete::TypeName;
use crate::type_checker::environment::expression_type::actual_type::ActualType;
use crate::type_checker::environment::expression_type::ExpressionType;
use crate::type_checker::type_result::TypeResult;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct MutableType {
    pub is_nullable: bool,
    pub is_mutable:  bool,
    pub actual_ty:   ActualType
}

impl Display for MutableType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let nullable = if self.is_nullable { "?" } else { "" };
        let mutable = if self.is_mutable { "mut " } else { "" };
        write!(f, "{}{}{}", mutable, self.actual_ty, nullable)
    }
}

impl From<&ActualType> for MutableType {
    fn from(actual_ty: &ActualType) -> Self {
        MutableType { is_nullable: false, is_mutable: false, actual_ty: actual_ty.clone() }
    }
}

impl MutableType {
    pub fn field(&self, field: &str, pos: &Position) -> TypeResult<Field> {
        self.actual_ty.field(field, pos)
    }

    pub fn fun(
        &self,
        name: &str,
        args: &[ExpressionType],
        safe: bool,
        pos: &Position
    ) -> TypeResult<Function> {
        let args: Vec<TypeName> =
            args.iter().map(|arg| self.expression_ty_to_type_name(arg)).collect();

        self.actual_ty.fun(name, &args, safe, pos)
    }

    fn expression_ty_to_type_name(&self, expr_ty: &ExpressionType) -> TypeName {
        match expr_ty {
            ExpressionType::Single { mut_ty } =>
                TypeName::Single { ty: self.asdf(&mut_ty.actual_ty) },
            ExpressionType::Union { union } => TypeName::Union {
                union: union.iter().map(|mut_ty| self.asdf(&mut_ty.actual_ty)).collect()
            }
        }
    }

    fn asdf(&self, actual_type: &ActualType) -> ActualTypeName {
        match actual_type {
            ActualType::Single { ty } => ty.name.clone(),
            ActualType::Tuple { types } => ActualTypeName::Tuple {
                ty_names: types.iter().map(|mut_ty| self.asdf(&mut_ty.actual_ty)).collect()
            },
            ActualType::AnonFun { args, ret_ty } => ActualTypeName::AnonFun {
                args:   args.iter().map(|mut_ty| self.asdf(&mut_ty.actual_ty)).collect(),
                ret_ty: Box::new(self.asdf(&ret_ty.actual_ty))
            }
        }
    }
}
