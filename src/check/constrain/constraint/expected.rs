use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::context::clss;
use crate::check::ty::name::TypeName;
use crate::common::delimit::comma_delimited;
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
                if let Some(stmt) = statements.last() {
                    stmt
                } else {
                    ast
                },
            _ => ast
        };

        Expected::new(&ast.pos, &Expression { ast: ast.clone() })
    }
}

impl From<&Box<AST>> for Expected {
    fn from(ast: &Box<AST>) -> Expected { Expected::from(ast.deref()) }
}

// TODO rework HasField

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expect {
    Statement,
    Nullable,
    Expression { ast: AST },
    ExpressionAny,
    Collection { ty: Box<Expected> },
    Truthy,
    Stringy,
    RaisesAny,
    Raises { raises: HashSet<TypeName> },
    Function { name: TypeName, args: Vec<Expected> },
    Field { name: String },
    Access { entity: Box<Expected>, name: Box<Expected> },
    Type { type_name: TypeName }
}

impl Hash for Expect {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            Raises { raises } => raises.iter().for_each(|t| t.hash(state)),
            Nullable => state.write_i8(1),
            Expression { ast } => ast.hash(state),
            ExpressionAny => state.write_i8(2),
            Collection { ty } => ty.hash(state),
            Truthy => state.write_i8(3),
            RaisesAny => state.write_i8(4),
            Stringy => state.write_i8(5),
            Statement => state.write_i8(6),
            Access { entity, name } => {
                entity.hash(state);
                name.hash(state);
            }
            Function { name, args } => {
                name.hash(state);
                args.iter().for_each(|a| a.hash(state));
            }
            Field { name } => name.hash(state),
            Type { type_name } => type_name.hash(state)
        }
    }
}

impl Display for Expect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match &self {
            Statement => String::from("()"),
            Nullable => String::from("None"),
            ExpressionAny => String::from("Any"),
            Expression { ast } => format!("{}", ast.node),
            Collection { ty } => format!("Collection[{}]", ty.expect),
            Truthy => format!("Truthy"),
            Stringy => format!("Stringy"),
            RaisesAny => String::from("RaisesAny"),
            Raises { raises: type_name } => format!("Raises[{{{}}}]", comma_delimited(type_name)),
            Access { entity, name } => format!("{}.{}", entity.expect, name.expect),
            Function { name, args } =>
                format!("{}({})", name, comma_delimited(args.iter().map(|e| e.expect.clone())),),
            Field { name } => name.clone(),
            Type { type_name } => format!("{}", type_name)
        })
    }
}

impl Expect {
    pub fn structurally_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Collection { ty: l }, Collection { ty: r }) => l.expect.structurally_eq(&r.expect),
            (Field { name: l }, Field { name: r }) => l == r,
            (Raises { raises: l }, Raises { raises: r }) => l == r,
            (Type { type_name: l }, Type { type_name: r }) => l == r,
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
            | (RaisesAny, RaisesAny)
            | (ExpressionAny, ExpressionAny)
            | (Stringy, Stringy)
            | (Statement, Statement)
            | (Nullable, Nullable) => true,

            (Truthy, Expression { ast: AST { node: Node::Bool { .. }, .. } })
            | (Expression { ast: AST { node: Node::Bool { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::And { .. }, .. } })
            | (Expression { ast: AST { node: Node::And { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::Or { .. }, .. } })
            | (Expression { ast: AST { node: Node::Or { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::Not { .. }, .. } })
            | (Expression { ast: AST { node: Node::Not { .. }, .. } }, Truthy) => true,
            (Type { type_name, .. }, Expression { ast: AST { node: Node::Str { .. }, .. } })
            | (Expression { ast: AST { node: Node::Str { .. }, .. } }, Type { type_name, .. })
                if type_name == &TypeName::from(clss::STRING_PRIMITIVE) =>
                true,
            (Type { type_name, .. }, Expression { ast: AST { node: Node::Real { .. }, .. } })
            | (Expression { ast: AST { node: Node::Real { .. }, .. } }, Type { type_name, .. })
                if type_name == &TypeName::from(clss::FLOAT_PRIMITIVE) =>
                true,
            (Type { type_name, .. }, Expression { ast: AST { node: Node::Int { .. }, .. } })
            | (Expression { ast: AST { node: Node::Int { .. }, .. } }, Type { type_name, .. })
                if type_name == &TypeName::from(clss::INT_PRIMITIVE) =>
                true,
            _ => false
        }
    }
}
