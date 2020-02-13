use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use crate::check::checker_result::{TypeErr, TypeResult};
use crate::check::context::ty::concrete;
use crate::check::ty::concrete::nullable::NullableType;
use crate::check::ty::name::actual::ActualTypeName;
use crate::check::ty::name::TypeName;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NullableTypeName {
    pub is_nullable: bool,
    pub actual:      ActualTypeName
}

impl Display for NullableTypeName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.actual,
            if self.is_nullable && self.actual != ActualTypeName::new(concrete::NONE, &[]) {
                "?"
            } else {
                ""
            }
        )
    }
}

impl TryFrom<&AST> for NullableTypeName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<NullableTypeName> {
        let (ast, nullable) = match &ast.node {
            Node::QuestionOp { expr } => (expr.deref(), true),
            _ => (ast, false)
        };

        Ok(NullableTypeName { is_nullable: nullable, actual: ActualTypeName::try_from(ast)? })
    }
}

impl From<&NullableType> for NullableTypeName {
    fn from(ty: &NullableType) -> Self {
        NullableTypeName {
            is_nullable: ty.is_nullable,
            actual:      ActualTypeName::from(&ty.actual_ty())
        }
    }
}

impl NullableTypeName {
    // TODO make more readable
    #[allow(clippy::nonminimal_bool)]
    pub fn is_superset(&self, other: &NullableTypeName) -> bool {
        self.is_nullable || (!self.is_nullable && !other.is_nullable) && self.actual == other.actual
    }

    pub fn substitute(
        &self,
        gens: &HashMap<String, TypeName>,
        pos: &Position
    ) -> TypeResult<NullableTypeName> {
        Ok(NullableTypeName {
            is_nullable: self.is_nullable,
            actual:      self.actual.substitute(gens, pos)?
        })
    }
}
