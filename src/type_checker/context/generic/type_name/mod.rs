use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::parser::ast::{Node, AST};
use crate::type_checker::context::generic::type_name::actual::GenericActualTypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub mod actual;

pub enum GenericTypeName {
    Single { ty: GenericActualTypeName },
    Union { union: HashSet<GenericActualTypeName> }
}

impl GenericTypeName {
    pub fn new(name: &str) -> GenericTypeName {
        GenericTypeName::Single { ty: GenericActualTypeName::new(name) }
    }
}
impl Display for GenericTypeName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GenericTypeName::Single { ty } => write!(f, "{}", ty),
            GenericTypeName::Union { union } => write!(f, "{{{:#?}}}", union)
        }
    }
}

impl TryFrom<&AST> for GenericTypeName {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericTypeName> {
        if let Node::TypeUnion { types } = &ast.node {
            let (types, errs) =
                types.iter().map(GenericTypeName::try_from).partition(Result::is_ok);
            if errs.is_empty() {
                Ok(GenericTypeName::Union {
                    union: types.into_iter().map(Result::unwrap).collect()
                })
            } else {
                Err(errs.into_iter().map(Result::unwrap_err).collect())
            }
        } else {
            GenericTypeName::Single {
                ty: GenericActualTypeName::try_from(ast).map_err(|e| vec![e])?
            }
        }
    }
}
