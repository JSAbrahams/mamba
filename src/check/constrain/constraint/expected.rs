use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::context::clss::{BOOL, FLOAT, INT, NONE, STRING};
use crate::check::name::{Name, Nullable};
use crate::check::name::string_name::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;
use crate::common::result::an_or_a;
use crate::parse::ast::{AST, Node};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Expected {
    pub pos: Position,
    pub expect: Expect,
    an_or_a: bool,
}

impl Expected {
    pub fn new(pos: Position, expect: &Expect) -> Expected {
        Expected { pos, expect: expect.clone(), an_or_a: true }
    }

    pub fn and_or_a(&self, and_or_a: bool) -> Expected {
        Expected { an_or_a: and_or_a, ..self.clone() }
    }
}

impl AsRef<Expected> for Expected {
    fn as_ref(&self) -> &Expected {
        self
    }
}

impl TryFrom<(&AST, &HashMap<String, String>)> for Expected {
    type Error = Vec<TypeErr>;

    /// Creates Expected from AST.
    ///
    /// If primitive or Constructor, constructs Type.
    fn try_from((ast, mappings): (&AST, &HashMap<String, String>)) -> TypeResult<Expected> {
        let ast = match &ast.node {
            Node::Block { statements } => statements.last().unwrap_or(ast),
            _ => ast,
        };

        Ok(Expected::new(ast.pos, &Expect::try_from((ast, mappings))?))
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
            Node::Int { .. } | Node::ENum { .. } => Type { name: Name::from(INT) },
            Node::Real { .. } => Type { name: Name::from(FLOAT) },
            Node::Bool { .. } => Type { name: Name::from(BOOL) },
            Node::Str { .. } => Type { name: Name::from(STRING) },
            Node::Undefined => Expect::none(),
            Node::Raise { error } => Raises { name: Name::try_from(error)? },
            _ => Expression { ast },
        })
    }
}

impl Display for Expected {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self.expect {
            Type { name } if self.an_or_a => write!(f, "{}{}", an_or_a(name), name),
            _ => write!(f, "{}", self.expect)
        }
    }
}

impl Display for Expect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            Expression { ast } => write!(f, "`{}`", ast.node),
            Collection { ty, .. } => write!(f, "{{{}}}", ty.and_or_a(false)),
            Tuple { elements } => {
                let elements: Vec<Expected> = elements.iter().map(|a| a.and_or_a(false)).collect();
                write!(f, "({})", comma_delm(elements))
            }
            Raises { name: ty } => write!(f, "Raises {}", ty),
            Access { entity, name } => write!(f, "{}.{}", entity.and_or_a(false), name.and_or_a(false)),
            Function { name, args } => {
                let args: Vec<Expected> = args.iter().map(|a| a.and_or_a(false)).collect();
                write!(f, "{}({})", name, comma_delm(args))
            }
            Field { name } => write!(f, "{}", name),
            Type { name } => write!(f, "{}", name),
        }
    }
}

impl Expect {
    /// True if same value.
    ///
    /// If other is a Raises or Type where the Name is temporary, and either is Raises in former case
    /// or Type in latter, then also true.
    pub fn same_value(&self, other: &Self) -> bool {
        match (self, other) {
            (Collection { ty: l }, Collection { ty: r }) => l.expect.same_value(&r.expect),
            (Field { name: l }, Field { name: r }) => l == r,
            (Raises { name: l }, Raises { name: r }) | (Type { name: l }, Type { name: r }) => {
                l == r
            }
            (Access { entity: le, name: ln }, Access { entity: re, name: rn }) => {
                le == re && ln == rn
            }
            (Function { name: l, args: la }, Function { name: r, args: ra }) => {
                l == r
                    && la.iter().zip_longest(ra.iter()).all(|pair| {
                    if let EitherOrBoth::Both(left, right) = pair {
                        left.expect.same_value(&right.expect)
                    } else {
                        false
                    }
                })
            }

            (Expression { ast: l }, Expression { ast: r }) => l.same_value(r),

            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Str { .. }, .. } })
            | (Expression { ast: AST { node: Node::Str { .. }, .. } }, Type { name: ty, .. })
            if ty == &Name::from(STRING) => {
                true
            }
            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Real { .. }, .. } })
            | (Expression { ast: AST { node: Node::Real { .. }, .. } }, Type { name: ty, .. })
            if ty == &Name::from(FLOAT) => {
                true
            }
            (Type { name: ty, .. }, Expression { ast: AST { node: Node::Int { .. }, .. } })
            | (Expression { ast: AST { node: Node::Int { .. }, .. } }, Type { name: ty, .. })
            if ty == &Name::from(INT) =>
                {
                    true
                }

            _ => self.is_none() && other.is_none(),
        }
    }

    pub fn none() -> Expect {
        Type { name: Name::from(NONE) }
    }

    pub fn is_none(&self) -> bool {
        match &self {
            Type { name } => name.is_null(),
            _ => false,
        }
    }
}
