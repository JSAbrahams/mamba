use std::fmt;
use std::fmt::{Display, Error, Formatter};
use std::hash::Hash;
use std::ops::Deref;

use itertools::{EitherOrBoth, Itertools};

use crate::check::constrain::constraint::builder::{format_var_map, VarMapping};
use crate::check::constrain::constraint::expected::Expect::*;
use crate::check::constrain::constraint::MapExp;
use crate::check::context::clss::NONE;
use crate::check::name::{Any, Name, Nullable};
use crate::check::name::string_name::StringName;
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

impl MapExp for Expected {
    fn map_exp(&self, var_mapping: &VarMapping, global_var_mapping: &VarMapping) -> Self {
        Expected::new(self.pos, &match &self.expect {
            Expression { ast } => {
                let ast = match &ast.node {
                    Node::Block { statements } if statements.is_empty() => ast.clone(),
                    Node::Block { statements } => statements.last().cloned().expect("unreachable"),
                    _ => ast.clone(),
                };

                Expression {
                    ast: ast.map(&|node: &Node| if let Node::Id { lit } = node {
                        let offset = if let Some(offset) = var_mapping.get(lit) {
                            *offset
                        } else if let Some(offset) = global_var_mapping.get(lit) {
                            *offset
                        } else {
                            0_usize
                        };

                        Node::Id { lit: format_var_map(lit, &offset) }
                    } else {
                        node.clone()
                    })
                }
            }
            Collection { ty } => Collection {
                ty: Box::from(ty.map_exp(var_mapping, global_var_mapping))
            },
            Tuple { elements } => Tuple {
                elements: elements.iter().map(|e| e.map_exp(var_mapping, global_var_mapping)).collect()
            },
            Function { name, args } => Function {
                name: name.clone(),
                args: args.iter().map(|a| a.map_exp(var_mapping, global_var_mapping)).collect(),
            },
            Access { entity, name } => Access {
                entity: Box::from(entity.map_exp(var_mapping, global_var_mapping)),
                name: Box::from(name.map_exp(var_mapping, global_var_mapping)),
            },
            other => other.clone()
        })
    }
}

impl Expected {
    pub fn new(pos: Position, expect: &Expect) -> Expected {
        Expected { pos, expect: expect.clone(), an_or_a: true }
    }

    pub fn and_or_a(&self, and_or_a: bool) -> Expected {
        Expected { an_or_a: and_or_a, ..self.clone() }
    }

    pub fn any(pos: Position) -> Expected {
        Expected::new(pos, &Type { name: Name::any() })
    }

    pub fn none(pos: Position) -> Expected {
        Expected::new(pos, &Type { name: Name::from(NONE) })
    }
}

impl AsRef<Expected> for Expected {
    fn as_ref(&self) -> &Expected {
        self
    }
}

impl From<&Box<AST>> for Expected {
    fn from(value: &Box<AST>) -> Self {
        Self::from(value.deref())
    }
}

impl From<&AST> for Expected {
    fn from(ast: &AST) -> Expected {
        Expected::new(ast.pos, &Expression { ast: ast.clone() })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expect {
    Expression { ast: AST },
    Collection { ty: Box<Expected> },
    Tuple { elements: Vec<Expected> },
    Function { name: StringName, args: Vec<Expected> },
    Field { name: String },
    Access { entity: Box<Expected>, name: Box<Expected> },
    Type { name: Name },
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
            Expression { ast } => write!(f, "{}", ast.node),
            Collection { ty, .. } => write!(f, "{{{}}}", ty.and_or_a(false)),
            Tuple { elements } => {
                let elements: Vec<Expected> = elements.iter().map(|a| a.and_or_a(false)).collect();
                write!(f, "({})", comma_delm(elements))
            }
            Access { entity, name } => write!(f, "{}.{}", entity.and_or_a(false), name.and_or_a(false)),
            Function { name, args } => {
                let args: Vec<Expected> = args.iter().map(|a| a.and_or_a(false)).collect();
                write!(f, "{}({})", name, comma_delm(args))
            }
            Field { name } => write!(f, "{name}"),
            Type { name } => write!(f, "{name}"),
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
            (Access { entity: le, name: ln }, Access { entity: re, name: rn }) => {
                le == re && ln == rn
            }
            (Type { name: l }, Type { name: r }) => l == r,

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

            _ => self.is_none() && other.is_none(),
        }
    }

    pub fn is_none(&self) -> bool {
        match &self {
            Type { name } => name.is_null(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::check::constrain::constraint::expected::{Expect, Expected};
    use crate::common::position::{CaretPos, Position};
    use crate::parse::ast::{AST, Node};
    use crate::parse::parse_direct;

    #[test]
    fn test_expected_from_int_constructor_call() {
        let ast = parse_direct("Int(10)").unwrap();
        let expect = Expected::from(&ast[0]);

        assert_eq!(expect.expect, Expect::Expression {
            ast: AST::new(
                Position::new(CaretPos::new(1, 1), CaretPos::new(1, 8)),
                Node::FunctionCall {
                    name: Box::new(AST::new(
                        Position::new(CaretPos::new(1, 1), CaretPos::new(1, 4)),
                        Node::Id { lit: String::from("Int") })),
                    args: vec![AST::new(
                        Position::new(CaretPos::new(1, 5), CaretPos::new(1, 7)),
                        Node::Int { lit: String::from("10") })],
                },
            )
        })
    }
}
