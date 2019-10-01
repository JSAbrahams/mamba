use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use crate::parser::ast::{Node, AST};
use crate::type_checker::type_result::{TypeErr, TypeResult};

mod actual;

pub enum GenericType {
    Single { ty: GenericType },
    Union { union: HashSet<GenericType> }
}

impl GenericType {
    pub fn new(name: &str) -> GenericType { GenericType::Single { ty: GenericType::new(name) } }
}
impl Display for GenericType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            GenericType::Single { ty } => write!(f, "{}", ty),
            GenericType::Union { union } => write!(f, "{{{:#?}}}", union)
        }
    }
}

impl TryFrom<&AST> for GenericType {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<GenericType> {
        if let Node::TypeUnion { types } = &ast.node {
            let (types, errs) = types.iter().map(GenericType::try_from).partition(Result::is_ok);
            if errs.is_empty() {
                Ok(types.into_iter().map(Result::unwrap).collect())
            } else {
                Err(errs.into_iter().map(Result::unwrap_err).collect())
            }
        } else {
            GenericType::try_from(ast).map_err(|e| vec![e])
        }
    }
}
