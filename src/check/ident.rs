use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::parse::ast::{Node, AST};

#[derive(Clone, Debug)]
pub struct Identifier {
    pub lit:   Option<(bool, String)>,
    pub names: Vec<Identifier>
}

impl Identifier {
    pub fn fields(&self) -> Vec<(bool, String)> {
        if let Some(lit) = &self.lit {
            vec![lit.clone()]
        } else {
            self.names.iter().map(|name| name.fields()).flatten().collect()
        }
    }

    pub fn as_mutable(&self, mutable: bool) -> Identifier {
        if let Some((_, id)) = &self.lit {
            Identifier { lit: Some((mutable, id.clone())), names: self.names.clone() }
        } else {
            Identifier {
                lit:   self.lit.clone(),
                names: self.names.iter().map(|name| name.as_mutable(mutable)).collect()
            }
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some((mutable, lit)) = &self.lit {
            write!(f, "{}{}", if *mutable { "mut " } else { "" }, lit.clone())
        } else {
            write!(f, "({})", comma_delm(&self.names))
        }
    }
}

impl TryFrom<&AST> for Identifier {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<Identifier> {
        match &ast.node {
            // TODO add mutable field to identifier
            Node::Id { lit } => Ok(Identifier::from((false, lit.as_str()))),
            Node::ExpressionType { expr, mutable, .. } => {
                let identifier = Identifier::try_from(expr.deref())?;
                Ok(identifier.as_mutable(*mutable))
            }
            Node::Tuple { elements } => {
                let elements =
                    elements.iter().map(Identifier::try_from).collect::<Result<_, _>>()?;
                Ok(Identifier::from(&elements))
            }
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected id or tuple of id's")])
        }
    }
}

impl From<&Vec<Identifier>> for Identifier {
    fn from(identifiers: &Vec<Identifier>) -> Self {
        Identifier { lit: None, names: identifiers.clone() }
    }
}

impl From<(bool, &str)> for Identifier {
    fn from((mutable, name): (bool, &str)) -> Self {
        // TODO use mutable field from identifier
        Identifier { lit: Some((mutable, String::from(name))), names: vec![] }
    }
}
