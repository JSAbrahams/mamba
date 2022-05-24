use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Identifier {
    Single(bool, IdentiCall),
    Multi(Vec<Identifier>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentiCall {
    Iden(String),
    Call(Box<IdentiCall>, Box<IdentiCall>),
}

impl IdentiCall {
    /// Recursively check Identical until [IdentiCall::Iden(String)] found in leftmost leaf.
    /// This means that object gives the top-level object if this is a chain of calls.
    ///
    /// ### Example
    ///
    /// An identifier representing a chain of calls `a.b.c` returns String "a".
    ///
    /// ### Errors
    ///
    /// Errors if no such left-most leaf exists.
    /// This is indicative of a malformed identifier, as all identifiers should have
    /// [IdentiCall::Iden] as leaves.
    pub fn object(&self, pos: &Position) -> TypeResult<String> {
        match &self.object_rec() {
            IdentiCall::Iden(object) => Ok(object.clone()),
            IdentiCall::Call(_, _) => Err(vec![TypeErr::new(pos, "Call not expected here")]),
        }
    }

    fn object_rec(&self) -> IdentiCall {
        match &self {
            IdentiCall::Iden(_) => self.clone(),
            IdentiCall::Call(instance, _) => instance.object_rec(),
        }
    }

    pub fn without_obj(&self, object: &str, pos: &Position) -> TypeResult<IdentiCall> {
        match &self {
            IdentiCall::Iden(_) => Err(vec![TypeErr::new(pos, "Call expected here")]),
            IdentiCall::Call(obj, call) => match obj.deref() {
                IdentiCall::Iden(str) if str == object => Ok(*call.clone()),
                IdentiCall::Iden(str) => {
                    let msg = format!("Call does not have identifier '{}'", str);
                    Err(vec![TypeErr::new(pos, &msg)])
                }
                obj => Ok(IdentiCall::Call(Box::from(obj.without_obj(object, pos)?), call.clone())),
            },
        }
    }
}

impl Identifier {
    pub fn all_calls(&self) -> Vec<IdentiCall> {
        match &self {
            Identifier::Single(_, call) => vec![call.clone()],
            Identifier::Multi(idens) => idens.iter().flat_map(|id| id.all_calls()).collect(),
        }
    }

    pub fn is_tuple(&self) -> bool {
        matches!(&self, Identifier::Multi(..))
    }

    pub fn fields(&self, pos: &Position) -> TypeResult<Vec<(bool, String)>> {
        match &self {
            Identifier::Single(mutable, call) => Ok(vec![(*mutable, call.object(pos)?)]),
            Identifier::Multi(ids) => {
                let ids: Vec<Vec<(bool, String)>> =
                    ids.iter()
                        .map(|id| id.fields(pos))
                        .collect::<TypeResult<Vec<Vec<(bool, String)>>>>()?;
                Ok(ids.into_iter().flatten().collect())
            }
        }
    }
}

impl Identifier {
    /// Make the top-level [IdentiCall] mutable, of both Single and Multi [Identifier]s.
    pub fn as_mutable(&self, mutable: bool) -> Identifier {
        match &self {
            Identifier::Single(_, name) => Identifier::Single(mutable, name.clone()),
            Identifier::Multi(idens) => {
                let idens = idens.iter().map(|id| id.as_mutable(mutable)).collect();
                Identifier::Multi(idens)
            }
        }
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            Identifier::Single(mutable, lit) => {
                write!(f, "{}{}", if *mutable { "" } else { "fin " }, lit.clone())
            }
            Identifier::Multi(ids) => write!(f, "({})", comma_delm(ids)),
        }
    }
}

impl Display for IdentiCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            IdentiCall::Iden(name) => write!(f, "{}", name),
            IdentiCall::Call(object, call) => write!(f, "{}.{}", object, call),
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
            _ => Err(vec![TypeErr::new(&ast.pos, "Expected id or tuple of id's")]),
        }
    }
}

impl From<&str> for IdentiCall {
    fn from(name: &str) -> Self {
        IdentiCall::Iden(String::from(name))
    }
}

impl From<&Vec<Identifier>> for Identifier {
    fn from(identifiers: &Vec<Identifier>) -> Self {
        Identifier::Multi(identifiers.clone())
    }
}

impl From<(bool, &str)> for Identifier {
    fn from((mutable, name): (bool, &str)) -> Self {
        Identifier::Single(mutable, IdentiCall::from(name))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::check::ident::{IdentiCall, Identifier};
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
    fn without_obj() -> TypeResult<()> {
        let call = IdentiCall::Call(
            Box::from(IdentiCall::Iden(String::from("a"))),
            Box::from(IdentiCall::Call(
                Box::from(IdentiCall::Iden(String::from("b"))),
                Box::from(IdentiCall::Iden(String::from("c"))),
            )),
        );

        assert_eq!(
            call.without_obj("a", &Position::default())?,
            IdentiCall::Call(
                Box::from(IdentiCall::Iden(String::from("b"))),
                Box::from(IdentiCall::Iden(String::from("c"))),
            )
        );
        Ok(())
    }

    #[test]
    fn without_obj_to_iden() -> TypeResult<()> {
        let call = IdentiCall::Call(
            Box::from(IdentiCall::Iden(String::from("a"))),
            Box::from(IdentiCall::Iden(String::from("b"))),
        );

        assert_eq!(
            call.without_obj("a", &Position::default())?,
            IdentiCall::Iden(String::from("b"))
        );
        Ok(())
    }

    #[test]
    fn from_expression_type() -> TypeResult<()> {
        let ast = AST::new(
            &Position::default(),
            Node::ExpressionType {
                expr: Box::new(AST::new(&Position::default(), Node::Id { lit: String::from("h") })),
                mutable: false,
                ty: None,
            },
        );
        let iden = Identifier::try_from(&ast)?;

        assert_eq!(iden, Identifier::from((false, "h")));
        Ok(())
    }

    #[test]
    fn from_expression_type_as_mutable() -> TypeResult<()> {
        let ast = AST::new(
            &Position::default(),
            Node::ExpressionType {
                expr: Box::new(AST::new(&Position::default(), Node::Id { lit: String::from("h") })),
                mutable: false,
                ty: None,
            },
        );
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
            ],
        };
        let ast = AST::new(&Position::default(), node);
        match Identifier::try_from(&ast)? {
            Identifier::Single(_, _) => panic!("Expected multi"),
            Identifier::Multi(idens) => {
                assert_eq!(idens.len(), 2);
                assert_eq!(idens[0], Identifier::from((true, "a")));
                assert_eq!(idens[1], Identifier::from((true, "b")));
            }
        }
        Ok(())
    }

    #[test]
    fn from_tuple_with_int_err() {
        let node = Node::Tuple {
            elements: vec![
                AST::new(&Position::default(), Node::Int { lit: String::from("a") }),
                AST::new(&Position::default(), Node::Id { lit: String::from("b") }),
            ],
        };
        let ast = AST::new(&Position::default(), node);
        let iden = Identifier::try_from(&ast);
        assert!(iden.is_err());
    }
}
