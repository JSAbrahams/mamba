use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use crate::check::context::clss::NONE;
use crate::check::result::{TypeErr, TypeResult};
use crate::check::ty::actual::ActualType;
use crate::check::ty::name::actual::ActualTypeName;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NullableType {
    pub is_nullable: bool,
    pub actual:      ActualTypeName
}

impl Display for NullableType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let display_nullable = self.is_nullable && self.actual != ActualType::new(NONE, &[]);
        write!(f, "{}{}", self.actual, if display_nullable { "?" } else { "" })
    }
}

impl TryFrom<&AST> for NullableType {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<NullableType> {
        match &ast.node {
            Node::QuestionOp { expr } =>
                Ok(NullableType { is_nullable: true, actual: ActualType::try_from(expr)? }),
            _ => Ok(NullableType { is_nullable: false, actual: ActualType::try_from(ast)? })
        }
    }
}

impl From<&NullableType> for NullableType {
    fn from(ty: &NullableType) -> Self {
        NullableType { is_nullable: ty.is_nullable, actual: ActualType::from(ty.actual) }
    }
}

impl NullableType {
    pub fn is_superset(&self, other: &NullableType) -> bool {
        let neither_nullable = !self.is_nullable && !other.is_nullable;
        self.is_nullable || neither_nullable && self.actual == other.actual
    }
}
