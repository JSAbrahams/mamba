use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::context::clss;
use crate::check::context::clss::{BOOL_PRIMITIVE, FLOAT_PRIMITIVE, INT_PRIMITIVE, NONE, STRING_PRIMITIVE};
use crate::check::name::Name;
use crate::check::name::stringname::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Expected {
    pub pos: Position,
    pub expect: Expect,
}

impl Expected {
    pub fn new(pos: &Position, expect: &Expect) -> Expected {
        Expected { pos: pos.clone(), expect: expect.clone() }
    }
}

impl AsRef<Expected> for Expected {
    fn as_ref(&self) -> &Expected { self }
}

impl TryFrom<(&AST, &HashMap<String, String>)> for Expected {
    type Error = Vec<TypeErr>;

    /// Creates Expected from AST.
    ///
    /// If primitive or Constructor, constructs Type.
    fn try_from((ast, mappings): (&AST, &HashMap<String, String>)) -> TypeResult<Expected> {
        let ast = match &ast.node {
            Node::Block { statements } => statements.last().unwrap_or(ast),
            _ => ast
        };

        Ok(Expected::new(&ast.pos, &Expect::try_from((ast, mappings))?))
    }
}

impl TryFrom<(&Box<AST>, &HashMap<String, String>)> for Expected {
    type Error = Vec<TypeErr>;

    fn try_from((ast, mappings): (&Box<AST>, &HashMap<String, String>)) -> TypeResult<Expected> {
        Expected::try_from((ast.deref(), mappings))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expect {
    Expression { ast: AST },
    ExpressionAny,
    Collection { ty: Box<Expected> },
    Tuple { elements: Vec<Expected> },
    Raises { name: Name },
    Function { name: StringName, args: Vec<Expected> },
    Field { name: String },
    Access { entity: Box<Expected>, name: Box<Expected> },
    Type { name: Name },
}

impl TryFrom<(&AST, &HashMap<String, String>)> for Expect {
    type Error = Vec<TypeErr>;

    /// Also substitutes any identifiers with new ones from the environment if the environment
    /// has a mapping.
    /// This means that we forget about shadowed variables and continue with the new ones.
    fn try_from((ast, mappings): (&AST, &HashMap<String, String>)) -> TypeResult<Expect> {
        let ast = ast.map(&|node: &Node| {
            if let Node::Id { lit } = node {
                if let Some(name) = mappings.get(lit) {
                    // Always use name currently defined in environment
                    Node::Id { lit: name.clone() }
                } else {
                    node.clone()
                }
            } else {
                node.clone()
            }
        });

        Ok(match &ast.node {
            Node::Int { .. } | Node::ENum { .. } => Type { name: Name::from(INT_PRIMITIVE) },
            Node::Real { .. } => Type { name: Name::from(FLOAT_PRIMITIVE) },
            Node::Bool { .. } => Type { name: Name::from(BOOL_PRIMITIVE) },
            Node::Str { .. } => Type { name: Name::from(STRING_PRIMITIVE) },
            Node::Undefined => Expect::none(),
            Node::Underscore => ExpressionAny,
            Node::Raise { error } => Raises { name: Name::try_from(error)? },
            _ => Expression { ast }
        })
    }
}

impl Display for Expected {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> { write!(f, "{}", self.expect) }
}

impl Display for Expect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match &self {
            ExpressionAny => String::from("Any"),
            Expression { ast } => format!("`{}`", ast.node),
            Collection { ty, .. } => format!("{{{}}}", ty.expect),
            Tuple { elements } => format!("({})", comma_delm(elements)),
            Raises { name: ty } => format!("Raises {}", ty),
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

            (ExpressionAny, ExpressionAny) => true,

            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Str { .. }, .. } })
            | (Expression { ast: AST { node: Node::Str { .. }, .. } }, Type { name: ty, .. })
            if ty == &Name::from(clss::STRING_PRIMITIVE) =>
                true,
            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Real { .. }, .. } })
            | (Expression { ast: AST { node: Node::Real { .. }, .. } }, Type { name: ty, .. })
            if ty == &Name::from(clss::FLOAT_PRIMITIVE) =>
                true,
            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Int { .. }, .. } })
            | (Expression { ast: AST { node: Node::Int { .. }, .. } }, Type { name: ty, .. })
            if ty == &Name::from(clss::INT_PRIMITIVE) =>
                true,

            _ => self.is_none() && other.is_none()
        }
    }

    pub fn none() -> Expect {
        Expect::Type { name: Name::from(NONE) }
    }

    pub fn is_none(&self) -> bool {
        match &self {
            Expect::Type { name } => name.is_null(),
            _ => false
        }
    }
}
