use std::fmt;
use std::fmt::{Display, Formatter};

use itertools::{EitherOrBoth, Itertools};

use crate::common::position::Position;
use crate::parser::ast::{Node, AST};
use crate::type_checker::constraints::constraint::expected::Expect::*;
use crate::type_checker::context::ty;
use crate::type_checker::type_name::TypeName;
use crate::type_checker::util::comma_delimited;
use std::ops::Deref;

#[derive(Clone, Debug, Eq)]
pub struct Expected {
    pub pos:    Position,
    pub expect: Expect
}

impl PartialEq for Expected {
    fn eq(&self, other: &Self) -> bool { self.expect == other.expect }
}

impl Expected {
    pub fn new(pos: &Position, expect: &Expect) -> Expected {
        Expected { pos: pos.clone(), expect: expect.clone() }
    }
}

impl From<&AST> for Expected {
    fn from(ast: &AST) -> Self { Expected::new(&ast.pos, &Expression { ast: ast.clone() }) }
}

impl From<&Box<AST>> for Expected {
    fn from(ast: &Box<AST>) -> Self { Expected::from(ast.deref()) }
}

// TODO rework HasField

#[derive(Clone, Debug, Eq)]
pub enum Expect {
    Nullable,
    Mutable,
    Expression { ast: AST },
    ExpressionAny,

    Collection { ty: Box<Expect> },
    Truthy,

    RaisesAny,
    Raises { type_name: TypeName },

    Implements { type_name: TypeName, args: Vec<Expected> },
    Function { name: TypeName, args: Vec<Expected> },
    HasField { name: String },

    Type { type_name: TypeName }
}

impl Display for Expect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match &self {
            Nullable => format!("None"),
            Mutable => format!("Mut"),
            ExpressionAny => String::from("Any"),
            Expression { ast } => format!("{:?}", ast.node),
            Collection { ty } => format!("[Collection[{}]]", ty),
            Truthy => format!("[{}]", ty::concrete::BOOL_PRIMITIVE),
            RaisesAny => String::from("[Raises[?]]"),
            Raises { type_name } => format!("[Raises[{}]]", type_name),
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

impl PartialEq for Expect {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Collection { ty: l }, Collection { ty: r }) => l == r,
            (HasField { name: l }, HasField { name: r }) => l == r,
            (Raises { type_name: l }, Raises { type_name: r })
            | (Type { type_name: l }, Type { type_name: r }) => l == r,

            (Implements { type_name: l, args: la }, Implements { type_name: r, args: ra })
            | (Function { name: l, args: la }, Function { name: r, args: ra }) =>
                l == r
                    && la.iter().zip_longest(ra.iter()).all(|pair| {
                        if let EitherOrBoth::Both(left, right) = pair {
                            left == right
                        } else {
                            false
                        }
                    }),

            (Expression { ast: l }, Expression { ast: r }) => l.equal_structure(r),
            (Truthy, Truthy) | (RaisesAny, RaisesAny) | (ExpressionAny, ExpressionAny) => true,

            (Truthy, Expression { ast: AST { node: Node::Bool { .. }, .. } })
            | (Expression { ast: AST { node: Node::Bool { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::And { .. }, .. } })
            | (Expression { ast: AST { node: Node::And { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::Or { .. }, .. } })
            | (Expression { ast: AST { node: Node::Or { .. }, .. } }, Truthy) => true,
            (Truthy, Expression { ast: AST { node: Node::Not { .. }, .. } })
            | (Expression { ast: AST { node: Node::Not { .. }, .. } }, Truthy) => true,

            (Type { type_name }, Expression { ast: AST { node: Node::Str { .. }, .. } })
            | (Expression { ast: AST { node: Node::Str { .. }, .. } }, Type { type_name })
                if type_name == &TypeName::from(ty::concrete::STRING_PRIMITIVE) =>
                true,
            (Type { type_name }, Expression { ast: AST { node: Node::Real { .. }, .. } })
            | (Expression { ast: AST { node: Node::Real { .. }, .. } }, Type { type_name })
                if type_name == &TypeName::from(ty::concrete::FLOAT_PRIMITIVE) =>
                true,
            (Type { type_name }, Expression { ast: AST { node: Node::Int { .. }, .. } })
            | (Expression { ast: AST { node: Node::Int { .. }, .. } }, Type { type_name })
                if type_name == &TypeName::from(ty::concrete::INT_PRIMITIVE) =>
                true,

            _ => false
        }
    }
}
