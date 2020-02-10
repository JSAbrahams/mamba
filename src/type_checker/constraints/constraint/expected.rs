use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::context::ty;
use crate::type_checker::ty_name::TypeName;
use crate::type_checker::util::comma_delimited;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

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
    Collection { ty: Box<Expect> },
    Truthy,
    Stringy,
    RaisesAny,
    Raises { raises: HashSet<TypeName> },
    Implements { type_name: TypeName, args: Vec<Expected> },
    Function { name: TypeName, args: Vec<Expected> },
    HasField { name: String },
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
            Implements { type_name, args } => {
                type_name.hash(state);
                args.iter().for_each(|a| a.hash(state))
            }
            Function { name, args } => {
                name.hash(state);
                args.iter().for_each(|a| a.hash(state));
            }
            HasField { name } => name.hash(state),
            Type { type_name } => type_name.hash(state)
        }
    }
}

impl Display for Expect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match &self {
            Statement => String::from("()"),
            Nullable => String::from("[None]"),
            ExpressionAny => String::from("[Any]"),
            Expression { ast } => format!("{}", ast.node),
            Collection { ty } =>
                if let Expression { .. } = &ty.deref() {
                    format!("[Collection{{{}}}]", ty)
                } else {
                    format!("[Collection{}]", ty)
                },
            Truthy => format!("[Truthy]"),
            Stringy => format!("[Stringy]"),
            RaisesAny => String::from("[RaisesAny]"),
            Raises { raises: type_name } => format!("[Raises{{{}}}]]", comma_delimited(type_name)),
            Implements { type_name, args } => format!(
                "[?.{}({})]",
                type_name,
                comma_delimited(args.iter().map(|e| e.expect.clone())),
            ),
            Function { name, args } =>
                format!("[{}({})]", name, comma_delimited(args.iter().map(|e| e.expect.clone())),),
            HasField { name } => format!("[?.{}]", name),
            Type { type_name } => format!("[{}]", type_name)
        })
    }
}

impl Expect {
    pub fn structurally_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Collection { ty: l }, Collection { ty: r }) => l.structurally_eq(r),
            (HasField { name: l }, HasField { name: r }) => l == r,
            (Raises { raises: l }, Raises { raises: r }) => l == r,
            (Type { type_name: l }, Type { type_name: r }) => l == r,
            (Implements { type_name: l, args: la }, Implements { type_name: r, args: ra })
            | (Function { name: l, args: la }, Function { name: r, args: ra }) =>
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
            | (Stringy, Stringy) => true,
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
                if type_name == &TypeName::from(ty::concrete::STRING_PRIMITIVE) =>
                true,
            (Type { type_name, .. }, Expression { ast: AST { node: Node::Real { .. }, .. } })
            | (Expression { ast: AST { node: Node::Real { .. }, .. } }, Type { type_name, .. })
                if type_name == &TypeName::from(ty::concrete::FLOAT_PRIMITIVE) =>
                true,
            (Type { type_name, .. }, Expression { ast: AST { node: Node::Int { .. }, .. } })
            | (Expression { ast: AST { node: Node::Int { .. }, .. } }, Type { type_name, .. })
                if type_name == &TypeName::from(ty::concrete::INT_PRIMITIVE) =>
                true,
            _ => false
        }
    }
}
