use std::fmt;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::context::clss;
use crate::check::context::name::{DirectName, NameUnion};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::parse::ast::{Node, AST};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Expected {
    pub pos:    Position,
    pub expect: Expect
}

impl Expected {
    pub fn new(pos: &Position, expect: &Expect) -> Expected {
        Expected { pos: pos.clone(), expect: expect.clone() }
    }
}

impl From<&AST> for Expected {
    fn from(ast: &AST) -> Expected {
        let ast = match &ast.node {
            Node::Block { statements } | Node::Script { statements } =>
                statements.last().unwrap_or(ast),
            _ => ast
        };
        Expected::new(&ast.pos, &Expression { ast: ast.clone() })
    }
}

impl From<&Box<AST>> for Expected {
    fn from(ast: &Box<AST>) -> Expected { Expected::from(ast.deref()) }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expect {
    Nullable,
    Expression { ast: AST },
    ExpressionAny,
    Collection { ty: Box<Expected> },
    Truthy,
    Stringy,
    Raises { name: NameUnion },
    Function { name: DirectName, args: Vec<Expected> },
    Field { name: String },
    Access { entity: Box<Expected>, name: Box<Expected> },
    Type { name: NameUnion }
}

impl Display for Expected {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> { write!(f, "{}", self.expect) }
}

impl Expected {
    pub fn is_expr(&self) -> bool {
        if let Expression { .. } = self.expect {
            true
        } else {
            false
        }
    }

    pub fn is_ty(&self) -> bool {
        if let Type { .. } = self.expect {
            true
        } else {
            false
        }
    }
}

impl Display for Expect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match &self {
            Nullable => String::from("None"),
            ExpressionAny => String::from("Any"),
            Expression { ast } => format!("{}", ast.node),
            Collection { ty } => format!("Collection[{}]", ty.expect),
            Truthy => format!("Truthy"),
            Stringy => format!("Stringy"),
            Raises { name: ty } => format!("Raises[{}]", ty),
            Access { entity, name } => format!("{}.{}", entity.expect, name.expect),
            Function { name, args } => format!("{}({})", name, comma_delm(args)),
            Field { name } => name.clone(),
            Type { name: ty } => format!("{}", ty)
        })
    }
}

impl Expect {
    pub fn structurally_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Collection { ty: l }, Collection { ty: r }) => l.expect.structurally_eq(&r.expect),
            (Field { name: l }, Field { name: r }) => l == r,
            (Raises { name: l }, Raises { name: r }) | (Type { name: l }, Type { name: r }) =>
                l == r,
            (Access { entity: le, name: ln }, Access { entity: re, name: rn }) =>
                le == re && ln == rn,
            (Function { name: l, args: la }, Function { name: r, args: ra }) =>
                l == r
                    && la.iter().zip_longest(ra.iter()).all(|pair| {
                        if let EitherOrBoth::Both(left, right) = pair {
                            left.expect.structurally_eq(&right.expect)
                        } else {
                            false
                        }
                    }),

            (Expression { ast: l }, Expression { ast: r }) => l.equal_structure(r),

            (Truthy, Truthy)
            | (ExpressionAny, ExpressionAny)
            | (Stringy, Stringy)
            | (Nullable, Nullable) => true,

            (Truthy, Expression { ast: AST { node: Node::Bool { .. }, .. } })
            | (Expression { ast: AST { node: Node::Bool { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::And { .. }, .. } })
            | (Expression { ast: AST { node: Node::And { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::Or { .. }, .. } })
            | (Expression { ast: AST { node: Node::Or { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::Not { .. }, .. } })
            | (Expression { ast: AST { node: Node::Not { .. }, .. } }, Truthy) => true,

            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Str { .. }, .. } })
            | (Expression { ast: AST { node: Node::Str { .. }, .. } }, Type { name: ty, .. })
                if ty == &NameUnion::from(clss::STRING_PRIMITIVE) =>
                true,
            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Real { .. }, .. } })
            | (Expression { ast: AST { node: Node::Real { .. }, .. } }, Type { name: ty, .. })
                if ty == &NameUnion::from(clss::FLOAT_PRIMITIVE) =>
                true,
            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Int { .. }, .. } })
            | (Expression { ast: AST { node: Node::Int { .. }, .. } }, Type { name: ty, .. })
                if ty == &NameUnion::from(clss::INT_PRIMITIVE) =>
                true,

            _ => false
        }
    }
}
