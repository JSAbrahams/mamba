use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    pub lit: Option<(bool, IdentiCall)>,
    pub names: Vec<Identifier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentiCall {
    object: String,
    pub property: Option<Box<IdentiCall>>,
}

impl IdentiCall {
    pub fn object(&self, pos: &Position) -> TypeResult<String> {
        if self.property.is_none() {
            Ok(self.object.clone())
        } else {
            Err(vec![TypeErr::new(pos, "Call not expected here")])
        }
    }
}

impl Identifier {
    pub fn is_tuple(&self) -> bool { self.names.len() > 1 }
}

impl Identifier {
    pub fn fields(&self) -> Vec<(bool, IdentiCall)> {
        if let Some(lit) = &self.lit {
            vec![lit.clone()]
        } else {
            self.names.iter().flat_map(|name| name.fields()).collect()
        }
    }

    pub fn as_mutable(&self, mutable: bool) -> Identifier {
        if let Some((_, id)) = &self.lit {
            Identifier { lit: Some((mutable, id.clone())), names: self.names.clone() }
        } else if mutable {
            self.clone()
        } else {
            // If not mutable, then make everything immutable
            Identifier {
                lit: self.lit.clone().map(|(_, str)| (false, str)),
                names: self.names.iter().map(|name| name.as_mutable(false)).collect(),
            }
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some((mutable, lit)) = &self.lit {
            write!(f, "{}{}", if *mutable { "" } else { "fin " }, lit.clone())
        } else {
            write!(f, "({})", comma_delm(&self.names))
        }
    }
}

impl Display for IdentiCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(call) = &self.property {
            write!(f, "{}.{}", self.object, call)
        } else {
            write!(f, "{}", self.object)
        }
    }
}

impl TryFrom<&AST> for Identifier {
    type Error = Vec<TypeErr>;

    fn try_from(ast: &AST) -> TypeResult<Identifier> {
        match &ast.node {
            Node::Id { lit } => Ok(Identifier::from((true, lit.as_str()))),
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

impl From<&str> for IdentiCall {
    fn from(name: &str) -> Self {
        IdentiCall { object: String::from(name), property: None }
    }
}

impl From<&Vec<Identifier>> for Identifier {
    fn from(identifiers: &Vec<Identifier>) -> Self {
        Identifier { lit: None, names: identifiers.clone() }
    }
}

impl From<(bool, &str)> for Identifier {
    fn from((mutable, name): (bool, &str)) -> Self {
        Identifier { lit: Some((mutable, IdentiCall::from(name))), names: vec![] }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::check::ident::Identifier;
    use crate::check::result::TypeResult;
    use crate::common::position::Position;
    use crate::parse::ast::{AST, Node};

    #[test]
    fn from_id() -> TypeResult<()> {
        let ast = AST::new(&Position::default(), Node::Id { lit: String::from("r") });
        let iden = Identifier::try_from(&ast)?;
        assert_eq!(iden, Identifier::from((true, "r")));
        Ok(())
    }

    #[test]
    fn from_expression_type() -> TypeResult<()> {
        let ast = AST::new(&Position::default(), Node::ExpressionType {
            expr: Box::new(AST::new(&Position::default(), Node::Id { lit: String::from("h") })),
            mutable: false,
            ty: None,
        });
        let iden = Identifier::try_from(&ast)?;

        assert_eq!(iden, Identifier::from((false, "h")));
        Ok(())
    }

    #[test]
    fn from_expression_type_as_mutable() -> TypeResult<()> {
        let ast = AST::new(&Position::default(), Node::ExpressionType {
            expr: Box::new(AST::new(&Position::default(), Node::Id { lit: String::from("h") })),
            mutable: false,
            ty: None,
        });
        let iden = Identifier::try_from(&ast)?.as_mutable(true);

        assert_eq!(iden, Identifier::from((true, "h")));
        Ok(())
    }

    #[test]
    fn from_int_error() {
        let ast = AST::new(&Position::default(), Node::Int { lit: String::from("r") });
        let res = Identifier::try_from(&ast);
        assert!(res.is_err())
    }

    #[test]
    fn from_tuple() -> TypeResult<()> {
        let node = Node::Tuple {
            elements: vec![
                AST::new(&Position::default(), Node::Id { lit: String::from("a") }),
                AST::new(&Position::default(), Node::Id { lit: String::from("b") }),
            ]
        };
        let ast = AST::new(&Position::default(), node);
        let iden = Identifier::try_from(&ast)?;

        assert!(iden.lit.is_none());
        let idens = iden.names;
        assert_eq!(idens.len(), 2);
        assert_eq!(idens[0], Identifier::from((true, "a")));
        assert_eq!(idens[1], Identifier::from((true, "b")));

        Ok(())
    }

    #[test]
    fn from_tuple_with_int_err() {
        let node = Node::Tuple {
            elements: vec![
                AST::new(&Position::default(), Node::Int { lit: String::from("a") }),
                AST::new(&Position::default(), Node::Id { lit: String::from("b") }),
            ]
        };
        let ast = AST::new(&Position::default(), node);
        let iden = Identifier::try_from(&ast);
        assert!(iden.is_err());
    }
}
